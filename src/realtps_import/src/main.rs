#![allow(unused)]

use anyhow::{Context, Result};
use calculate::ChainCalcs;
use client::{Client, EthersClient, NearClient, SolanaClient, TendermintClient};
use futures::stream::{FuturesUnordered, StreamExt};
use log::{error, info};
use realtps_common::{all_chains, Chain, Db, JsonDb};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::task;
use tokio::task::JoinHandle;

mod calculate;
mod client;
mod delay;
mod import;

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(StructOpt, Debug)]
enum Command {
    Run,
    Import,
    Calculate,
}

enum Job {
    Import(Chain),
    Calculate,
}

static RPC_CONFIG_PATH: &str = "rpc_config.toml";

#[derive(Deserialize, Serialize)]
struct RpcConfig {
    chains: HashMap<Chain, String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let opts = Opts::from_args();
    let cmd = opts.cmd.unwrap_or(Command::Run);

    let rpc_config = load_rpc_config(RPC_CONFIG_PATH)?;

    Ok(run(cmd, rpc_config).await?)
}

async fn run(cmd: Command, rpc_config: RpcConfig) -> Result<()> {
    let importer = make_importer(&rpc_config).await?;

    let mut jobs = FuturesUnordered::new();

    for job in init_jobs(cmd).into_iter() {
        jobs.push(importer.do_job(job));
    }

    loop {
        let job_result = jobs.next().await;
        if let Some(new_jobs) = job_result {
            for new_job in new_jobs {
                jobs.push(importer.do_job(new_job));
            }
        } else {
            error!("no more jobs?!");
            break;
        }
    }

    Ok(())
}

fn load_rpc_config<P: AsRef<Path>>(path: P) -> Result<RpcConfig> {
    let rpc_config_file = fs::read_to_string(path).context("unable to load RPC configuration")?;
    let rpc_config = toml::from_str::<RpcConfig>(&rpc_config_file)
        .context("unable to parse RPC configuration")?;

    Ok(rpc_config)
}

fn print_error(e: &anyhow::Error) {
    error!("error: {}", e);
    let mut source = e.source();
    while let Some(source_) = source {
        error!("source: {}", source_);
        source = source_.source();
    }
}

fn init_jobs(cmd: Command) -> Vec<Job> {
    match cmd {
        Command::Run => {
            let import_jobs = init_jobs(Command::Import);
            let calculate_jobs = init_jobs(Command::Calculate);
            import_jobs
                .into_iter()
                .chain(calculate_jobs.into_iter())
                .collect()
        }
        Command::Import => all_chains().into_iter().map(Job::Import).collect(),
        Command::Calculate => {
            vec![Job::Calculate]
        }
    }
}

async fn make_importer(rpc_config: &RpcConfig) -> Result<Importer> {
    let clients = make_all_clients(rpc_config).await?;

    Ok(Importer {
        db: Arc::new(JsonDb),
        clients,
    })
}

async fn make_all_clients(rpc_config: &RpcConfig) -> Result<HashMap<Chain, Box<dyn Client>>> {
    let mut client_futures = vec![];
    for chain in all_chains() {
        let rpc_url = get_rpc_url(&chain, rpc_config).to_string();
        let client_future = task::spawn(make_client(chain, rpc_url));
        client_futures.push((chain, client_future));
    }

    let mut clients = HashMap::new();

    for (chain, client_future) in client_futures {
        let client = client_future.await??;
        clients.insert(chain, client);
    }

    Ok(clients)
}

async fn make_client(chain: Chain, rpc_url: String) -> Result<Box<dyn Client>> {
    info!("creating client for {} at {}", chain, rpc_url);
    let client: Box<dyn Client>;

    match chain {
        Chain::Arbitrum
            | Chain::Avalanche
            | Chain::Binance
            | Chain::Celo
            | Chain::Cronos
            | Chain::Ethereum
            | Chain::Fuse
            | Chain::Fantom
            | Chain::Harmony
            | Chain::Heco
            | Chain::KuCoin
            | Chain::Moonriver
            | Chain::OKEx
            | Chain::Polygon
            | Chain::Rootstock
            | Chain::Telos
            | Chain::XDai => client = Box::new(EthersClient::new(chain, &rpc_url)?),
        Chain::CosmosHub
            | Chain::SecretNetwork
            | Chain::Terra => client = Box::new(TendermintClient::new(chain, &rpc_url)?),
        Chain::Near => client = Box::new(NearClient::new(&rpc_url)?),
        Chain::Solana => client = Box::new(SolanaClient::new(&rpc_url)?),
    }
    let version = client.client_version().await?;
    info!("node version for {}: {}", chain, version);

    Ok(client)
}

fn get_rpc_url<'a>(chain: &Chain, rpc_config: &'a RpcConfig) -> &'a str {
    if let Some(url) = rpc_config.chains.get(chain) {
        return url;
    } else {
        todo!()
    }
}

struct Importer {
    db: Arc<dyn Db>,
    clients: HashMap<Chain, Box<dyn Client>>,
}

impl Importer {
    async fn do_job(&self, job: Job) -> Vec<Job> {
        let r = match job {
            Job::Import(chain) => self.import(chain).await,
            Job::Calculate => self.calculate().await,
        };

        match r {
            Ok(new_jobs) => new_jobs,
            Err(e) => {
                print_error(&e);
                error!("error running job. repeating");
                delay::job_error_delay().await;
                vec![job]
            }
        }
    }

    async fn import(&self, chain: Chain) -> Result<Vec<Job>> {
        let client = self.clients.get(&chain).expect("client");
        import::import(chain, client.as_ref(), &self.db).await?;
        Ok(vec![Job::Import(chain)])
    }

    async fn calculate(&self) -> Result<Vec<Job>> {
        info!("beginning tps calculation");
        let tasks: Vec<(Chain, JoinHandle<Result<ChainCalcs>>)> = all_chains()
            .into_iter()
            .map(|chain| {
                let calc_future = calculate::calculate_for_chain(chain, self.db.clone());
                (chain, task::spawn(calc_future))
            })
            .collect();

        for (chain, task) in tasks {
            let res = task.await?;
            match res {
                Ok(calcs) => {
                    info!("calculated {} tps for chain {}", calcs.tps, calcs.chain);
                    let db = self.db.clone();
                    task::spawn_blocking(move || db.store_tps(calcs.chain, calcs.tps)).await??;
                }
                Err(e) => {
                    print_error(&anyhow::Error::from(e));
                    error!("error calculating for {}", chain);
                }
            }
        }

        delay::recalculate_delay().await;

        Ok(vec![Job::Calculate])
    }
}

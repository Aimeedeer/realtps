#![allow(unused)]

use anyhow::{anyhow, Context, Result};
use ethers::prelude::*;
use ethers::utils::hex::ToHex;
use futures::stream::{FuturesUnordered, StreamExt};
use rand::{
    self,
    distributions::{Distribution, Uniform},
};
use realtps_common::{Block, Chain, Db, JsonDb};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::task;
use tokio::time::{self, Duration};
use tokio::task::JoinHandle;

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
            println!("no more jobs?!");
            break;
        }
    }

    Ok(())
}

fn load_rpc_config<P: AsRef<Path>>(path: P) -> Result<RpcConfig> {
    let rpc_config_file = fs::read_to_string(path)?;
    let rpc_config =
        toml::from_str::<RpcConfig>(&rpc_config_file).context("parsing RPC configuration")?;

    Ok(rpc_config)
}

fn print_error(e: &anyhow::Error) {
    eprintln!("error: {}", e);
    let mut source = e.source();
    while let Some(source_) = source {
        eprintln!("source: {}", source_);
        source = source_.source();
    }
}

fn all_chains() -> Vec<Chain> {
    vec![
        Chain::Ethereum,
        Chain::Polygon,
        Chain::Avalanche,
        Chain::Celo,
        Chain::Fantom,
        Chain::Moonriver,
        Chain::Arbitrum,
        Chain::Binance,
        Chain::Harmony,
        Chain::Rootstock,
    ]
}

fn init_jobs(cmd: Command) -> Vec<Job> {
    match cmd {
        Command::Run | Command::Import => {
            all_chains().into_iter().map(Job::Import).collect()
        }
        Command::Calculate => {
            vec![
                Job::Calculate,
            ]
        }
    }
}

async fn make_importer(rpc_config: &RpcConfig) -> Result<Importer> {
    let mut eth_providers = HashMap::new();
    for chain in all_chains() {
        let provider = make_provider(chain, get_rpc_url(&chain, rpc_config)).await?;
        eth_providers.insert(chain, provider);
    }

    Ok(Importer {
        db: Arc::new(Box::new(JsonDb)),
        eth_providers,
    })
}

fn get_rpc_url<'a>(chain: &Chain, rpc_config: &'a RpcConfig) -> &'a str {
    if let Some(url) = rpc_config.chains.get(chain) {
        return url;
    } else {
        todo!()
    }
}

async fn make_provider(chain: Chain, rpc_url: &str) -> Result<Provider<Http>> {
    println!("creating ethers provider for {} at {}", chain, rpc_url);

    let provider = Provider::<Http>::try_from(rpc_url)?;

    let version = provider.client_version().await?;
    println!("node version: {}", version);

    Ok(provider)
}

struct Importer {
    db: Arc<Box<dyn Db>>,
    eth_providers: HashMap<Chain, Provider<Http>>,
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
                println!("error running job. repeating");
                job_error_delay().await;
                vec![job]
            }
        }
    }

    async fn import(&self, chain: Chain) -> Result<Vec<Job>> {
        println!("beginning import for {}", chain);

        let provider = self.provider(chain);
        let head_block_number = provider.get_block_number().await?;
        let head_block_number = head_block_number.as_u64();
        println!("head block number: {}", head_block_number);

        let highest_block_number = self.db.load_highest_block_number(chain)?;

        if Some(head_block_number) != highest_block_number {
            let initial_sync = highest_block_number.is_none();
            const INITIAL_SYNC_BLOCKS: u64 = 100;
            let mut synced = 0;

            let mut block_number = head_block_number;

            loop {
                println!("fetching block {} for {}", block_number, chain);

                let ethers_block_number = U64::from(block_number);

                let block = loop {
                    let block = provider.get_block(ethers_block_number).await?;

                    if let Some(block) = block {
                        break block;
                    } else {
                        println!(
                            "received no block for number {} on chain {}",
                            block_number, chain
                        );
                        retry_delay().await;
                    }
                };

                let block = ethers_block_to_block(chain, block)?;

                let parent_hash = block.parent_hash.clone();

                let db = self.db.clone();
                task::spawn_blocking(move || db.store_block(block)).await??;

                synced += 1;

                if initial_sync && synced == INITIAL_SYNC_BLOCKS {
                    println!("finished initial sync for {}", chain);
                    break;
                }

                if let Some(prev_block_number) = block_number.checked_sub(1) {
                    let db = self.db.clone();
                    let prev_block =
                        task::spawn_blocking(move || db.load_block(chain, prev_block_number))
                            .await??;

                    if let Some(prev_block) = prev_block {
                        if prev_block.hash != parent_hash {
                            println!(
                                "reorg of chain {} at block {}; old hash: {}; new hash: {}",
                                chain, prev_block_number, prev_block.hash, parent_hash
                            );
                            // continue - have wrong version of prev block
                        } else {
                            if let Some(highest_block_number) = highest_block_number {
                                if prev_block_number <= highest_block_number {
                                    println!(
                                        "completed import of chain {} to block {} / {}",
                                        chain, prev_block_number, parent_hash
                                    );
                                    break;
                                } else {
                                    println!(
                                        "found incomplete previous import for {} at block {}",
                                        chain, prev_block_number
                                    );
                                    // Found a run of blocks from a previous incomplete import.
                                    // Keep going and overwrite them.
                                    // continue
                                }
                            } else {
                                println!(
                                    "found incomplete previous import for {} at block {}",
                                    chain, prev_block_number
                                );
                                // Found a run of blocks from a previous incomplete import.
                                // Keep going and overwrite them.
                                // continue
                            }
                        }
                    } else {
                        // continue - don't have the prev block
                    }

                    println!("still need block {} for {}", prev_block_number, chain);
                    block_number = prev_block_number;

                    courtesy_delay().await;

                    continue;
                } else {
                    println!("completed import of chain {} to genesis", chain);
                    break;
                }
            }

            let db = self.db.clone();
            task::spawn_blocking(move || db.store_highest_block_number(chain, head_block_number))
                .await??;
        } else {
            println!("no new blocks for {}", chain);
        }

        rescan_delay(chain).await;

        Ok(vec![Job::Import(chain)])
    }

    fn provider(&self, chain: Chain) -> &Provider<Http> {
        self.eth_providers.get(&chain).expect("provider")
    }

    async fn calculate(&self) -> Result<Vec<Job>> {
        let tasks: Vec<(Chain, JoinHandle<Result<ChainCalcs>>)> = all_chains().into_iter().map(|chain| {
            let calc_future = calculate_for_chain(self.db.clone(), chain);
            (chain, task::spawn(calc_future))
        }).collect();

        for (chain, task) in tasks {
            let res = task.await?;
            match res {
                Ok(calcs) => {
                    println!("calculated {} tps for chain {}", calcs.tps, calcs.chain);
                    let db = self.db.clone();
                    task::spawn_blocking(move || {
                        db.store_tps(calcs.chain, calcs.tps)
                    }).await??;
                }
                Err(e) => {
                    print_error(&anyhow::Error::from(e));
                    println!("error calculating for {}", chain);
                }
            }
        }

        recalculate_delay().await;

        Ok(vec![Job::Calculate])
    }
}

struct ChainCalcs {
    chain: Chain,
    tps: f64,
}

async fn calculate_for_chain(db: Arc<Box<dyn Db>>, chain: Chain) -> Result<ChainCalcs> {
    let highest_block_number = {
        let db = db.clone();
        task::spawn_blocking(move || {
            db.load_highest_block_number(chain)
        }).await??
    };
    let highest_block_number = highest_block_number.ok_or_else(|| anyhow!("no data for chain {}", chain))?;

    async fn load_block_(db: &Arc<Box<dyn Db>>, chain: Chain, number: u64) -> Result<Option<Block>> {
        let db = db.clone();
        task::spawn_blocking(move || {
            db.load_block(chain, number)
        }).await?
    }

    let load_block = |number| load_block_(&db, chain, number);

    let latest_timestamp = load_block(highest_block_number).await?.expect("first block").timestamp;

    let seconds_per_week = 60 * 60 * 24 * 7;
    let min_timestamp = latest_timestamp.checked_sub(seconds_per_week).expect("underflow");

    let mut current_block_number = highest_block_number;
    let mut current_block = load_block(current_block_number).await?.expect("first_block");

    let mut num_txs: u64 = 0;

    let init_timestamp = loop {
        assert!(current_block_number != 0);

        let prev_block_number = current_block_number - 1;
        let prev_block = load_block(prev_block_number).await?;

        if let Some(prev_block) = prev_block {
            num_txs = num_txs.checked_add(current_block.num_txs).expect("overflow");

            assert!(prev_block.timestamp <= current_block.timestamp);

            if prev_block.timestamp <= min_timestamp {
                break prev_block.timestamp;
            }
            if prev_block.block_number == 0 {
                break prev_block.timestamp;
            }

            current_block_number = prev_block_number;
            current_block = prev_block;
        } else {
            break current_block.timestamp;
        }
    };

    assert!(init_timestamp <= latest_timestamp);
    let total_seconds = latest_timestamp - init_timestamp;
    let total_seconds_u32 = u32::try_from(total_seconds).map_err(|_| anyhow!("seconds overflows u32"))?;
    let num_txs_u32 = u32::try_from(num_txs).map_err(|_| anyhow!("num txs overflows u32"))?;
    let total_seconds_f64 = f64::from(total_seconds_u32);
    let num_txs_f64 = f64::from(num_txs_u32);
    let tps = num_txs_f64 / total_seconds_f64;

    Ok(ChainCalcs {
        chain,
        tps,
    })
}

fn ethers_block_to_block(chain: Chain, block: ethers::prelude::Block<H256>) -> Result<Block> {
    Ok(Block {
        chain,
        block_number: block.number.expect("block number").as_u64(),
        timestamp: u64::try_from(block.timestamp).map_err(|e| anyhow!("{}", e))?,
        num_txs: u64::try_from(block.transactions.len())?,
        hash: block.hash.expect("hash").encode_hex(),
        parent_hash: block.parent_hash.encode_hex(),
    })
}

async fn delay(base_ms: u64) {
    let jitter = Uniform::from(0..100);
    let delay_msecs = base_ms + jitter.sample(&mut rand::thread_rng());
    let delay_time = Duration::from_millis(delay_msecs);
    time::sleep(delay_time).await
}

async fn courtesy_delay() {
    let msecs = 100;
    println!("delaying {} ms to retrieve next block", msecs);
    delay(msecs).await
}

async fn rescan_delay(chain: Chain) {
    let delay_secs = match chain {
        _ => 30, /* todo */
    };
    let msecs = 1000 * delay_secs;
    println!("delaying {} ms to {} rescan", msecs, chain);
    delay(msecs).await
}

async fn retry_delay() {
    let msecs = 100;
    println!("delaying {} ms to retry request", msecs);
    delay(msecs).await
}

async fn job_error_delay() {
    let msecs = 1000;
    println!("delaying {} ms to retry job", msecs);
    delay(msecs);
}

async fn recalculate_delay() {
    let msecs = 1000;
    println!("delaying {} ms before recaclulating", msecs);
    delay(msecs);
}

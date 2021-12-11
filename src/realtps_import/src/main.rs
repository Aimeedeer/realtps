#![allow(unused)]

use anyhow::{Result, anyhow};
use structopt::StructOpt;
use ethers::prelude::*;
use std::sync::Arc;
use std::collections::VecDeque;
use std::collections::HashMap;

use realtps_common::{Chain, Block, Db, JsonDb};

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    ReadBlock { number: u64 },
}

enum Job {
    ImportMostRecent(Chain),
    ImportBlock(Chain, u64),
}

#[tokio::main]
async fn main() -> Result<()> {

    let importer = make_importer().await?;

    let mut jobs = VecDeque::from(init_jobs());

    while let Some(job) = jobs.pop_front() {
        let new_jobs = importer.do_job(job).await?;
        jobs.extend(new_jobs.into_iter());
    }
    
    Ok(())
}

fn init_jobs() -> Vec<Job> {
    vec![
        Job::ImportMostRecent(Chain::Ethereum),
        Job::ImportMostRecent(Chain::Polygon),
    ]
}

async fn make_importer() -> Result<Importer> {
    let eth_providers = [
        (Chain::Ethereum, make_provider(get_rpc_url(Chain::Ethereum)).await?),
        (Chain::Polygon, make_provider(get_rpc_url(Chain::Polygon)).await?),
    ];

    Ok(Importer {
        db: Box::new(JsonDb),
        eth_providers: eth_providers.into_iter().collect(),
    })
}

static ETHEREUM_MAINNET_RPC: &str = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27";
static POLYGON_MAINNET_RPC: &str = "https://polygon-rpc.com/";

fn get_rpc_url(chain: Chain) -> &'static str {
    match chain {
        Chain::Ethereum => ETHEREUM_MAINNET_RPC,
        Chain::Polygon => POLYGON_MAINNET_RPC,
    }
}

async fn make_provider(rpc_url: &str) -> Result<Provider<Http>> {
    println!("creating ethers provider for {}", rpc_url);

    let provider = Provider::<Http>::try_from(rpc_url)?;

    let version = provider.client_version().await?;
    println!("node version: {}", version);

    Ok(provider)
}

struct Importer {
    db: Box<dyn Db>,
    eth_providers: HashMap<Chain, Provider<Http>>,
}

impl Importer {
    async fn do_job(&self, job: Job) -> Result<Vec<Job>> {
        match job {
            Job::ImportMostRecent(chain) => {
                let block_num = self.get_current_block(chain).await?;
                Ok(self.import_block(chain, block_num).await?)
            },
            Job::ImportBlock(chain, block_num) => {
                Ok(self.import_block(chain, block_num).await?)
            }
        }
    }

    async fn get_current_block(&self, chain: Chain) -> Result<u64> {
        todo!()
    }

    async fn import_block(&self, chain: Chain, block_num: u64) -> Result<Vec<Job>> {
        todo!()
    }
}

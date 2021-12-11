#![allow(unused)]

use anyhow::{anyhow, Result};
use ethers::prelude::*;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::task;
use realtps_common::{Block, Chain, Db, JsonDb};
use tokio::time::{self, Duration};
use rand::{self, distributions::{Uniform, Distribution}};

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
        (
            Chain::Ethereum,
            make_provider(get_rpc_url(Chain::Ethereum)).await?,
        ),
        (
            Chain::Polygon,
            make_provider(get_rpc_url(Chain::Polygon)).await?,
        ),
    ];

    Ok(Importer {
        db: Arc::new(Box::new(JsonDb)),
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
    db: Arc<Box<dyn Db>>,
    eth_providers: HashMap<Chain, Provider<Http>>,
}

impl Importer {
    async fn do_job(&self, job: Job) -> Result<Vec<Job>> {
        match job {
            Job::ImportMostRecent(chain) => {
                let block_num = self.get_current_block(chain).await?;
                Ok(self.import_block(chain, block_num).await?)
            }
            Job::ImportBlock(chain, block_num) => Ok(self.import_block(chain, block_num).await?),
        }
    }

    async fn get_current_block(&self, chain: Chain) -> Result<u64> {
        let provider = self.provider(chain);
        let block_number = provider.get_block_number().await?;
        println!("block number: {}", block_number);
        Ok(block_number.as_u64())
    }

    async fn import_block(&self, chain: Chain, block_num: u64) -> Result<Vec<Job>> {
        println!("importing block {} for chain {}", block_num, chain);
        let provider = self.provider(chain);
        let ethers_block_num = U64::from(block_num);
        let block = provider.get_block(ethers_block_num).await?.expect("block");

        let block = Block {
            chain,
            block_number: block_num,
            timestamp: u64::try_from(block.timestamp).map_err(|e| anyhow!("{}", e))?,
            num_txs: u64::try_from(block.transactions.len())?,
            hash: format!("{}", block.hash.expect("hash")),
            parent_hash: format!("{}", block.parent_hash),
        };

        let db = self.db.clone();

        let next_jobs = task::spawn_blocking(move || -> Result<_> {
            let parent_hash = block.parent_hash.clone();

            db.store_block(block)?;

            if let Some(prev_block_num) = block_num.checked_sub(1) {
                let prev_block = db.load_block(prev_block_num)?;

                if let Some(prev_block) = prev_block {
                    if prev_block.hash != parent_hash {
                        println!("reorg of chain {} at block {}; old hash: {}; new hash: {}",
                                 chain, prev_block_num, parent_hash, prev_block.hash);
                        Ok(vec![Job::ImportBlock(chain, prev_block_num)])
                    } else {
                        println!("completed import of chain {} to block {} / {}",
                                 chain, prev_block_num, parent_hash);
                        Ok(vec![])
                    }
                } else {
                    Ok(vec![Job::ImportBlock(chain, prev_block_num)])
                }
            } else {
                println!("completed import of chain {} to genesis", chain);
                Ok(vec![])
            }
        }).await??;

        if !next_jobs.is_empty() {
            courtesy_delay().await;
            Ok(next_jobs)
        } else {
            rescan_delay(chain).await;
            Ok(vec![Job::ImportMostRecent(chain)])
        }
    }

    fn provider(&self, chain: Chain) -> &Provider<Http> {
        self.eth_providers.get(&chain).expect("provider")
    }
}

async fn courtesy_delay() {
    let jitter = Uniform::from(0..100);
    let delay_msecs = 1000 + jitter.sample(&mut rand::thread_rng());
    println!("delaying {} ms to retrieve next block", delay_msecs);
    let delay_time = Duration::from_millis(delay_msecs);
    time::sleep(delay_time).await
}

async fn rescan_delay(chain: Chain) {
    let delay_secs = match chain {
        Chain::Ethereum => 60,
        Chain::Polygon => 10,
    };
    let jitter = Uniform::from(0..100);
    let delay_msecs = 1000 * delay_secs + jitter.sample(&mut rand::thread_rng());
    let delay_time = Duration::from_millis(delay_msecs);
    println!("delaying {} ms to rescan", delay_msecs);
    time::sleep(delay_time).await
}

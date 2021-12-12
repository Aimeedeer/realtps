#![allow(unused)]

use anyhow::{anyhow, Result};
use ethers::prelude::*;
use rand::{
    self,
    distributions::{Distribution, Uniform},
};
use realtps_common::{Block, Chain, Db, JsonDb};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::task;
use tokio::time::{self, Duration};

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
    Import(Chain),
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
    vec![Job::Import(Chain::Ethereum), Job::Import(Chain::Polygon)]
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
            Job::Import(chain) => Ok(self.import(chain).await?),
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
                let block = provider
                    .get_block(ethers_block_number)
                    .await?
                    .expect("block");
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
}

async fn courtesy_delay() {
    let jitter = Uniform::from(0..100);
    let delay_msecs = 100 + jitter.sample(&mut rand::thread_rng());
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

fn ethers_block_to_block(chain: Chain, block: ethers::prelude::Block<H256>) -> Result<Block> {
    Ok(Block {
        chain,
        block_number: block.number.expect("block number").as_u64(),
        timestamp: u64::try_from(block.timestamp).map_err(|e| anyhow!("{}", e))?,
        num_txs: u64::try_from(block.transactions.len())?,
        hash: format!("{}", block.hash.expect("hash")),
        parent_hash: format!("{}", block.parent_hash),
    })
}

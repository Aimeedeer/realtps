#![allow(unused)]

use anyhow::{Result, anyhow};
use structopt::StructOpt;
use ethers::prelude::*;
use std::sync::Arc;
use std::collections::VecDeque;

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

    let importer = Importer { };

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

struct Importer {
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

use crate::calculate;
use crate::calculate::ChainCalcs;
use crate::client::Client;
use crate::delay;
use crate::import;
use anyhow::Result;
use futures::future::FutureExt;
use futures::stream::FuturesUnordered;
use log::{error, info};
use realtps_common::{Chain, Db};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::task;
use tokio::task::JoinHandle;

pub enum Job {
    Import(Chain),
    Calculate(Vec<Chain>),
}

pub struct JobRunner {
    pub db: Arc<dyn Db>,
    pub clients: HashMap<Chain, Box<dyn Client>>,
}

impl JobRunner {
    pub async fn do_job(&self, job: Job) -> Vec<Job> {
        let r = match job {
            Job::Import(chain) => self.import(chain).await,
            Job::Calculate(ref chains) => self.calculate(chains.to_vec()).await,
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

    async fn calculate(&self, chains: Vec<Chain>) -> Result<Vec<Job>> {
        info!("beginning tps calculation");

        let start = Instant::now();

        let tasks: FuturesUnordered<JoinHandle<(Chain, Result<ChainCalcs>)>> = chains
            .iter()
            .map(|chain| {
                let chain = *chain;
                let calc_future = calculate::calculate_for_chain(chain, self.db.clone());
                let calc_future = calc_future.map(move |r| (chain, r));
                task::spawn(calc_future)
            })
            .collect();

        for task in tasks {
            let (chain, res) = task.await?;
            match res {
                Ok(calcs) => {
                    info!("calculated {} tps for chain {}", calcs.tps, calcs.chain);
                    let db = self.db.clone();
                    task::spawn_blocking(move || db.store_tps(calcs.chain, calcs.tps)).await??;
                }
                Err(e) => {
                    print_error(&e);
                    error!("error calculating for {}", chain);
                }
            }
        }

        let end = Instant::now();
        let duration = end - start;
        info!("calculation took {} s", duration.as_secs());

        delay::recalculate_delay().await;

        Ok(vec![Job::Calculate(chains)])
    }
}

fn print_error(e: &anyhow::Error) {
    error!("error: {}", e);
    let mut source = e.source();
    while let Some(source_) = source {
        error!("source: {}", source_);
        source = source_.source();
    }
}

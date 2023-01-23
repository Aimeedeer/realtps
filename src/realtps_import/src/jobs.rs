use crate::calculate;
use crate::client::Client;
use crate::delay;
use crate::import;
use crate::remove;
use anyhow::{Context, Result};
use futures::future::FutureExt;
use futures::stream::{FuturesUnordered, StreamExt};
use log::{error, info};
use rand::prelude::*;
use realtps_common::{chain::Chain, db::Db};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::task;

#[derive(Debug)]
pub enum Job {
    Import(Chain),
    Calculate(Vec<Chain>),
    Remove(Vec<Chain>),
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
            Job::Remove(ref chains) => self.remove(chains.to_vec()).await,
        };

        match r {
            Ok(new_jobs) => new_jobs,
            Err(e) => {
                print_error(&e);
                error!("error running job: {}. repeating", &e);
                delay::job_error_delay(&job).await;
                vec![job]
            }
        }
    }

    async fn import(&self, chain: Chain) -> Result<Vec<Job>> {
        let client = self
            .clients
            .get(&chain)
            .context(format!("no client for {}", chain))?;
        import::import(chain, client.as_ref(), &self.db).await?;

        Ok(vec![Job::Import(chain)])
    }

    async fn calculate(&self, chains: Vec<Chain>) -> Result<Vec<Job>> {
        info!("beginning tps calculation");

        let start = Instant::now();

        let mut tasks: FuturesUnordered<_> = chains
            .iter()
            .map(|chain| {
                let chain = *chain;
                let calc_future = calculate::calculate_for_chain(chain, self.db.clone());
                let calc_future = task::spawn(calc_future);
                calc_future.map(move |calcs| (chain, calcs))
            })
            .collect();

        while let Some((chain, calcs)) = tasks.next().await {
            let calcs = calcs?;
            match calcs {
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

    async fn remove(&self, chains: Vec<Chain>) -> Result<Vec<Job>> {
        info!("removing old data");

        let mut rng = rand::thread_rng();
        let mut chains = chains;
        chains.shuffle(&mut rng);

        for chain in &chains {
            remove::remove_old_data_for_chain(*chain, self.db.clone()).await?;
        }

        delay::remove_data_delay().await;

        Ok(vec![Job::Remove(chains)])
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

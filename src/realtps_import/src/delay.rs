use crate::Chain;
use crate::Job;
use anyhow::Result;
use log::{debug, warn};
use rand::{
    self,
    distributions::{Distribution, Uniform},
};
use std::future::Future;
use std::pin::Pin;
use tokio::time::{self, Duration};

/// The default rate to request blocks at, in ms.
const DEFAULT_BLOCK_PACE: u64 = 500;

/// The default time to wait between imports, in ms.
const DEFAULT_RESCAN_DELAY: u64 = 30000;

/// The pace we want to request blocks at, in ms.
pub fn block_pace(chain: Chain) -> u64 {
    match chain {
        Chain::Arbitrum => 400, // Subsecond block time
        Chain::Bitcoin => 2000,
        Chain::MultiversX => 1000,   // 6s block time
        Chain::Optimism => 2000, // Got blocked at 1000ms, unclear what rate they want
        // Need to go fast to keep up.
        // Solana's RpcClient will use its built in rate limiter when connecting to public nodes.
        Chain::Solana => 0,
        _ => DEFAULT_BLOCK_PACE,
    }
}

/// Wait between imports, in s.
///
/// This should be somewhat longer than the average block production time (or
/// perhaps the block production time / 2) to avoid making requests for new
/// blocks when there are none, but low enough that the block pace can catch up
/// to new blocks.
pub async fn rescan_delay(chain: Chain) {
    let delay_msecs = match chain {
        Chain::Arbitrum => 5000, // Subsecond block time
        Chain::Bitcoin => 600000,
        Chain::Hedera => 10000,
        Chain::Kusama => 7000,    // Like Polkadot
        Chain::Optimism => 15000, // Unclear, just experimenting
        Chain::Pivx => 5000,
        Chain::Polkadot => 7000, // 6s block time, server rate-limited, can't wait too long
        Chain::Solana => 1000,   // Need to go fast to keep up
        _ => DEFAULT_RESCAN_DELAY,
    };

    debug!("delaying {} ms to rescan chain {}", delay_msecs, chain);
    delay(delay_msecs).await
}

async fn delay(base_ms: u64) {
    let jitter = Uniform::from(0..10);
    let delay_msecs = base_ms + jitter.sample(&mut rand::thread_rng());
    let delay_time = Duration::from_millis(delay_msecs);
    time::sleep(delay_time).await;
}

pub async fn job_error_delay(job: &Job) {
    let msecs = 1000;
    debug!("delaying {} ms to retry job {:?}", msecs, job);
    delay(msecs).await;
}

pub async fn recalculate_delay() {
    let msecs = 5000;
    debug!("delaying {} ms before recaclulating", msecs);
    delay(msecs).await;
}

pub async fn remove_data_delay() {
    let msecs = 60 * 60 * 24 * 1000;
    debug!("delaying {} ms to remove old blocks", msecs);
    delay(msecs).await;
}

pub async fn retry_if_err<'caller, F, T>(chain: Chain, f: F) -> Result<T>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'caller>>,
{
    let tries = 3;
    let base_delay_ms = 500;
    let mut try_num = 1;

    loop {
        let r = f().await;
        match r {
            Ok(r) => break Ok(r),
            Err(e) => {
                if try_num == tries {
                    break Err(e);
                } else {
                    let delay_ms = base_delay_ms * try_num;
                    warn!(
                        "for chain {} received err {}. retrying in {} ms",
                        chain, e, delay_ms
                    );
                    delay(delay_ms).await;
                }
            }
        }
        try_num += 1;
    }
}

pub async fn retry_if_none<'caller, F, T>(chain: Chain, f: F) -> Result<Option<T>>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<Option<T>>> + Send + 'caller>>,
{
    let tries = 3;
    let base_delay_ms = 500;
    let mut try_num = 1;

    loop {
        let r = f().await?;
        match r {
            Some(r) => break Ok(Some(r)),
            None => {
                if try_num == tries {
                    break Ok(None);
                } else {
                    let delay_ms = base_delay_ms * try_num;
                    warn!(
                        "for chain {} received None. retrying in {} ms",
                        chain, delay_ms
                    );
                    delay(delay_ms).await;
                }
            }
        }
        try_num += 1;
    }
}

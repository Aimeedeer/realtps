use crate::Chain;
use log::debug;
use rand::{
    self,
    distributions::{Distribution, Uniform},
};
use tokio::time::{self, Duration};

async fn delay(base_ms: u64) {
    let jitter = Uniform::from(0..10);
    let delay_msecs = base_ms + jitter.sample(&mut rand::thread_rng());
    let delay_time = Duration::from_millis(delay_msecs);
    time::sleep(delay_time).await;
}

pub async fn courtesy_delay(chain: Chain) {
    let msecs = match chain {
        // Need to go fast to keep up.
        // Solana's RpcClient will use its built in rate limiter when connecting to public nodes.
        Chain::Solana => 0,
        _ => 200,
    };
    debug!("delaying {} ms to retrieve next block", msecs);
    delay(msecs).await
}

pub async fn rescan_delay(chain: Chain) {
    let delay_secs = match chain {
        Chain::Solana => 1, // Need to go fast to keep up
        _ => 30,
    };
    let msecs = 1000 * delay_secs;
    debug!("delaying {} ms to rescan", msecs);
    delay(msecs).await
}

pub async fn retry_delay() {
    let msecs = 100;
    debug!("delaying {} ms to retry request", msecs);
    delay(msecs).await
}

pub async fn job_error_delay() {
    let msecs = 1000;
    debug!("delaying {} ms to retry job", msecs);
    delay(msecs).await;
}

pub async fn recalculate_delay() {
    let msecs = 5000;
    debug!("delaying {} ms before recaclulating", msecs);
    delay(msecs).await;
}

use crate::delay;
use crate::Chain;
use log::debug;
use std::time::Instant;
use tokio::time::{self, Duration};

pub struct PaceSetter {
    chain: Chain,
    last_time: Instant,
}

impl PaceSetter {
    pub fn new(chain: Chain) -> Self {
        PaceSetter {
            chain,
            last_time: Instant::now(),
        }
    }

    pub async fn wait(&mut self) -> &mut Self {
        let work_duration = Instant::now().duration_since(self.last_time);
        let block_pace = Duration::from_millis(delay::block_pace(self.chain));

        if let Some(to_delay) = block_pace.checked_sub(work_duration) {
            debug!(
                "chain {} delaying {} ms to retrieve next block, block_pace {} ms",
                self.chain,
                to_delay.as_millis(),
                block_pace.as_millis(),
            );
            time::sleep(to_delay).await;
        } else {
            debug!(
                "chain {} worked {} ms, {} ms longer than block_pace {} ms",
                self.chain,
                work_duration.as_millis(),
                work_duration
                    .checked_sub(block_pace)
                    .expect("overflow")
                    .as_millis(),
                block_pace.as_millis(),
            );
        }

        self.last_time = Instant::now();
        self
    }
}

use crate::delay;
use crate::Chain;
use log::{debug, trace};
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
        let courtesy_delay = Duration::from_millis(delay::courtesy_delay(self.chain));

        if let Some(to_delay) = courtesy_delay.checked_sub(work_duration) {
            debug!(
                "chain {} delaying {} ms to retrieve next block",
                self.chain.description(),
                to_delay.as_millis()
            );
            time::sleep(to_delay).await;
        } else {
            trace!(
                "chain {} works {} ms longer than courtesy delay",
                self.chain.description(),
                work_duration
                    .checked_sub(courtesy_delay)
                    .expect("overflow")
                    .as_millis()
            );
        }

        self.last_time = Instant::now();
        self
    }
}

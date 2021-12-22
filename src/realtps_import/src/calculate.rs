use anyhow::{anyhow, Result};
use log::{debug, warn};
use realtps_common::{Block, Chain, Db};
use std::sync::Arc;
use std::time;
use tokio::task;

pub struct ChainCalcs {
    pub chain: Chain,
    pub tps: f64,
}

pub async fn calculate_for_chain(chain: Chain, db: Arc<dyn Db>) -> Result<ChainCalcs> {
    let highest_block_number = {
        let db = db.clone();
        task::spawn_blocking(move || db.load_highest_block_number(chain)).await??
    };
    let highest_block_number =
        highest_block_number.ok_or_else(|| anyhow!("no data for chain {}", chain))?;

    async fn load_block_(db: &Arc<dyn Db>, chain: Chain, number: u64) -> Result<Option<Block>> {
        let db = db.clone();
        task::spawn_blocking(move || db.load_block(chain, number)).await?
    }

    let load_block = |number| load_block_(&db, chain, number);

    let latest_timestamp = load_block(highest_block_number)
        .await?
        .expect("first block")
        .timestamp;

    let seconds_per_week = 60 * 60 * 24 * 7;
    let min_timestamp = latest_timestamp
        .checked_sub(seconds_per_week)
        .expect("underflow");

    let mut current_block_number = highest_block_number;
    let mut current_block = load_block(current_block_number)
        .await?
        .expect("first_block");

    let mut num_txs: u64 = 0;

    let start = time::Instant::now();

    let mut blocks = 0;

    let init_timestamp = loop {
        let now = time::Instant::now();
        let duration = now - start;
        let secs = duration.as_secs();
        if secs > 0 {
            debug!("bps for {}: {:.2}", chain, blocks as f64 / secs as f64)
        }
        blocks += 1;

        assert!(current_block_number != 0);

        let prev_block_number = current_block.prev_block_number;
        if let Some(prev_block_number) = prev_block_number {
            let prev_block = load_block(prev_block_number).await?;

            if let Some(prev_block) = prev_block {
                num_txs = num_txs
                    .checked_add(current_block.num_txs)
                    .expect("overflow");

                if prev_block.timestamp > current_block.timestamp {
                    warn!(
                        "non-monotonic timestamp in block {} for chain {}. prev: {}; current: {}",
                        current_block_number, chain, prev_block.timestamp, current_block.timestamp
                    );
                }

                if prev_block.timestamp <= min_timestamp {
                    break prev_block.timestamp;
                }
                if prev_block.block_number == 0 {
                    break prev_block.timestamp;
                }

                current_block_number = prev_block_number;
                current_block = prev_block;
            } else {
                break current_block.timestamp;
            }
        } else {
            break current_block.timestamp;
        }
    };

    assert!(init_timestamp <= latest_timestamp);
    let total_seconds = latest_timestamp - init_timestamp;
    let total_seconds_u32 =
        u32::try_from(total_seconds).map_err(|_| anyhow!("seconds overflows u32"))?;
    let num_txs_u32 = u32::try_from(num_txs).map_err(|_| anyhow!("num txs overflows u32"))?;
    let total_seconds_f64 = f64::from(total_seconds_u32);
    let num_txs_f64 = f64::from(num_txs_u32);
    let mut tps = num_txs_f64 / total_seconds_f64;

    // Special float values will not serialize sensibly
    if tps.is_nan() || tps.is_infinite() {
        tps = 0.0;
    }

    Ok(ChainCalcs { chain, tps })
}

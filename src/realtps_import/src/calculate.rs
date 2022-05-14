use crate::helpers::*;
use anyhow::{anyhow, Result};
use log::info;
use realtps_common::{chain::Chain, db::Db};
use std::sync::Arc;
use std::time::SystemTime;

pub struct ChainCalcs {
    pub chain: Chain,
    pub tps: f64,
}

pub async fn calculate_for_chain(chain: Chain, db: Arc<dyn Db>) -> Result<ChainCalcs> {
    let start_processing_timestamp = SystemTime::now();

    let highest_block_number = load_highest_known_block_number(chain, &db).await?;
    let highest_block_number =
        highest_block_number.ok_or_else(|| anyhow!("no data for chain {}", chain))?;

    let load_block = |number| load_block(chain, &db, number);

    let latest_timestamp = load_block(highest_block_number)
        .await?
        .expect("first block")
        .timestamp;

    let seconds_per_week = 60 * 60 * 24 * 7;
    let min_timestamp = latest_timestamp
        .checked_sub(seconds_per_week)
        .expect("underflow");

    let mut current_block = load_block(highest_block_number)
        .await?
        .expect("first_block");

    let mut num_txs: u64 = 0;

    let init_timestamp = loop {
        let prev_block_number = current_block.prev_block_number;

        if prev_block_number.is_none() {
            break current_block.timestamp;
        }

        let prev_block_number = prev_block_number.unwrap();

        let prev_block = load_block(prev_block_number).await?;

        if prev_block.is_none() {
            break current_block.timestamp;
        }

        let prev_block = prev_block.unwrap();

        num_txs = num_txs
            .checked_add(current_block.num_txs)
            .expect("overflow");

        if prev_block.timestamp <= min_timestamp {
            break prev_block.timestamp;
        }
        if prev_block.block_number == 0 {
            break prev_block.timestamp;
        }

        current_block = prev_block;
    };

    let tps = calculate_tps(init_timestamp, latest_timestamp, num_txs)?;

    let end_processing_timestamp = SystemTime::now();

    let calculate_log = format!(
        "done calculation for chain {}:
processing start at: {:#?} and end at {:#?}.
timestamp of the newest block: {},
timestamp of the oldest block: {}",
        chain,
        start_processing_timestamp,
        end_processing_timestamp,
        latest_timestamp,
        init_timestamp,
    );
    info!("{}", calculate_log);
    write_log(chain, &db, calculate_log).await?;

    Ok(ChainCalcs { chain, tps })
}

fn calculate_tps(init_timestamp: u64, latest_timestamp: u64, num_txs: u64) -> Result<f64> {
    let total_seconds = latest_timestamp.saturating_sub(init_timestamp);
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

    Ok(tps)
}

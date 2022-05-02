use crate::client::Client;
use crate::delay::{retry_if_err, retry_if_none};
use anyhow::{anyhow, Result};
use log::debug;
use realtps_common::{
    chain::Chain,
    db::{Block, Db},
};
use std::sync::Arc;
use tokio::task;

pub async fn fetch_live_head_block_number(chain: Chain, client: &dyn Client) -> Result<u64> {
    let live_head_block_number =
        retry_if_err(chain, || Box::pin(client.get_latest_block_number())).await?;

    debug!(
        "live head block number for {}: {}",
        chain, live_head_block_number
    );

    Ok(live_head_block_number)
}

pub async fn fetch_block(chain: Chain, client: &dyn Client, block_number: u64) -> Result<Block> {
    debug!("fetching block {} for {}", block_number, chain);

    let get_block = || retry_if_err(chain, || Box::pin(client.get_block(block_number)));
    let maybe_block = retry_if_none(chain, || Box::pin(get_block())).await?;
    let block =
        maybe_block.ok_or_else(|| anyhow!("get block returned None for chain {}", chain))?;

    Ok(block)
}

pub async fn store_highest_known_block_number(
    chain: Chain,
    db: &Arc<dyn Db>,
    block_number: u64,
) -> Result<()> {
    let db = db.clone();
    task::spawn_blocking(move || db.store_highest_block_number(chain, block_number)).await??;

    debug!(
        "new highest known block number for {}: {}",
        chain, block_number
    );

    Ok(())
}

pub async fn load_highest_known_block_number(
    chain: Chain,
    db: &Arc<dyn Db>,
) -> Result<Option<u64>> {
    let db = db.clone();
    let highest_known_block_number =
        task::spawn_blocking(move || db.load_highest_block_number(chain)).await??;

    debug!(
        "highest known block number for {}: {:?}",
        chain, highest_known_block_number
    );

    Ok(highest_known_block_number)
}

pub async fn store_block(db: &Arc<dyn Db>, block: Block) -> Result<()> {
    let db = db.clone();
    task::spawn_blocking(move || db.store_block(block)).await??;
    Ok(())
}

pub async fn load_block(
    chain: Chain,
    db: &Arc<dyn Db>,
    block_number: u64,
) -> Result<Option<Block>> {
    let db = db.clone();
    let block = task::spawn_blocking(move || db.load_block(chain, block_number)).await??;
    Ok(block)
}

pub async fn remove_blocks(chain: Chain, db: &Arc<dyn Db>, blocks: Vec<u64>) -> Result<()> {
    let db = db.clone();
    task::spawn_blocking(move || db.remove_blocks(chain, blocks)).await??;
    Ok(())
}

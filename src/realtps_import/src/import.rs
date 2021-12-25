use crate::client::Client;
use crate::delay::{self, retry_if_err, retry_if_none};
use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use realtps_common::{Block, Chain, Db};
use std::sync::Arc;
use tokio::task;

pub async fn import(chain: Chain, client: &dyn Client, db: &Arc<dyn Db>) -> Result<()> {
    let res = import_no_rescan_delay(chain, client, db).await;

    match res {
        Ok(res) => {
            delay::rescan_delay(chain).await;
            Ok(res)
        }
        Err(e) => {
            // Delay will be handled by general error handler
            Err(e)
        }
    }
}

async fn import_no_rescan_delay(chain: Chain, client: &dyn Client, db: &Arc<dyn Db>) -> Result<()> {
    info!("beginning import for {}", chain);

    let highest_known_block_number = load_highest_known_block_number(chain, db).await?;
    let live_head_block_number = fetch_live_head_block_number(chain, client).await?;

    // If we've never synced this chain before, then just establish the first
    // few blocks, and the highest_known_block_number, and wait until next time.
    {
        let first_import = highest_known_block_number.is_none();
        if first_import {
            import_first_blocks(chain, client, db, live_head_block_number).await?;
            return Ok(());
        }
    }

    // todo: this and the above could be a let-else expr
    let highest_known_block_number = highest_known_block_number.unwrap();

    if live_head_block_number == highest_known_block_number {
        info!("no new blocks for chain {}", chain);
        return Ok(());
    } else if live_head_block_number < highest_known_block_number {
        warn!("live_head_block_number < highest_known_block_number for chain {}. head: {}; highest: {}",
              chain, live_head_block_number, highest_known_block_number);
        return Ok(());
    } else {
        let needed_blocks = live_head_block_number
            .checked_sub(highest_known_block_number)
            .expect("underflow");
        info!("importing at least {} blocks for {}", needed_blocks, chain);
    }

    sync(
        chain,
        client,
        db,
        highest_known_block_number,
        live_head_block_number,
    )
    .await?;

    Ok(())
}

/// Fetches and stores blocks starting from `live_head_block_number`, working
/// backwards until it reaches `highest_known_block_number`, accounting for
/// chain reorgs, and missing blocks from previous imports, and finally storing
/// a new highest known block number to disk.
async fn sync(
    chain: Chain,
    client: &dyn Client,
    db: &Arc<dyn Db>,
    highest_known_block_number: u64,
    live_head_block_number: u64,
) -> Result<()> {
    // todo: this doesn't check whether the blocks we're receiving have hash
    // chains that are consistent - we could be in the middle of a reorg, or get
    // conflicting info from different nodes behind a load balancer. The latter
    // case could leave us with blocks that aren't actually in the chain.

    let mut block_number = live_head_block_number;
    let joined_chain_block_number;
    let joined_chain_block_hash;

    loop {
        let block = fetch_block(chain, client, block_number).await?;
        let prev_block_number = block.prev_block_number.expect("not genesis block");
        let prev_block_hash = block.parent_hash.clone();

        store_block(db, block).await?;

        let prev_stored_block = load_block(chain, db, prev_block_number).await?;

        // If we already have the block then we need to decide whether we have
        // completed the import back to the previous highest_known_block_number,
        // whether the previous known block hash disagrees with the newly
        // fetched previous block hash, and whether we already have the previous
        // block from a previous import that failed to complete.

        let block_number_to_fetch_next = if let Some(prev_stored_block) = prev_stored_block {
            let chain_reorg = prev_stored_block.hash != prev_block_hash;
            if !chain_reorg {
                if prev_block_number <= highest_known_block_number {
                    // We did it!
                    joined_chain_block_number = prev_block_number;
                    joined_chain_block_hash = prev_block_hash;
                    break;
                } else {
                    // This is a block we've seen before, but it has a higher block
                    // number than our highest_known_block. This indicates a previous
                    // incomplete import. To avoid wasting a lot of time and bandwidth
                    // "fast-forward" through all the blocks we already know.
                    let highest_unknown_block = fast_forward(chain, db, prev_stored_block).await?;
                    highest_unknown_block
                }
            } else {
                warn!(
                    "reorg of chain {} at block {}; old hash: {}; new hash: {}",
                    chain, prev_block_number, prev_stored_block.hash, prev_block_hash
                );
                // continue - have wrong version of prev block
                prev_block_number
            }
        } else {
            prev_block_number
        };

        debug!(
            "still need block {} for {}",
            block_number_to_fetch_next, chain
        );
        block_number = block_number_to_fetch_next;

        delay::courtesy_delay(chain).await;
    }

    store_highest_known_block_number(chain, db, live_head_block_number).await?;

    info!(
        "completed import of chain {} to block {} / {}",
        chain, joined_chain_block_number, joined_chain_block_hash
    );

    Ok(())
}

async fn fetch_live_head_block_number(chain: Chain, client: &dyn Client) -> Result<u64> {
    let live_head_block_number =
        retry_if_err(|| Box::pin(client.get_latest_block_number())).await?;

    debug!(
        "live head block number for {}: {}",
        chain, live_head_block_number
    );

    Ok(live_head_block_number)
}

async fn store_highest_known_block_number(
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

async fn load_highest_known_block_number(chain: Chain, db: &Arc<dyn Db>) -> Result<Option<u64>> {
    let db = db.clone();
    let highest_known_block_number =
        task::spawn_blocking(move || db.load_highest_block_number(chain)).await??;

    debug!(
        "highest known block number for {}: {:?}",
        chain, highest_known_block_number
    );

    Ok(highest_known_block_number)
}

async fn import_first_blocks(
    chain: Chain,
    client: &dyn Client,
    db: &Arc<dyn Db>,
    head_block_number: u64,
) -> Result<()> {
    info!("importing first block for {}", chain);

    let head_block = fetch_block(chain, client, head_block_number).await?;
    let prev_block_number = head_block.prev_block_number.expect("not genesis block");
    let prev_block_hash = head_block.parent_hash.clone();
    let prev_block = fetch_block(chain, client, prev_block_number).await?;

    if prev_block_hash != prev_block.hash {
        // Immediate reorg. We'll just let the job scheduler try again.
        return Err(anyhow!("first blocks' hashes don't match for {}", chain));
    }

    store_block(db, head_block).await?;
    store_block(db, prev_block).await?;
    store_highest_known_block_number(chain, db, head_block_number).await?;

    Ok(())
}

async fn fetch_block(chain: Chain, client: &dyn Client, block_number: u64) -> Result<Block> {
    debug!("fetching block {} for {}", block_number, chain);

    let get_block = || retry_if_err(|| Box::pin(client.get_block(block_number)));
    let maybe_block = retry_if_none(|| Box::pin(get_block())).await?;
    let block =
        maybe_block.ok_or_else(|| anyhow!("get block returned None for chain {}", chain))?;

    Ok(block)
}

async fn store_block(db: &Arc<dyn Db>, block: Block) -> Result<()> {
    let db = db.clone();
    task::spawn_blocking(move || db.store_block(block)).await??;
    Ok(())
}

async fn load_block(chain: Chain, db: &Arc<dyn Db>, block_number: u64) -> Result<Option<Block>> {
    let db = db.clone();
    let block = task::spawn_blocking(move || db.load_block(chain, block_number)).await??;
    Ok(block)
}

/// Starting from a known good block, fast-forward until we see a black with a
/// hash mismatch, or that we don't have yet.
async fn fast_forward(chain: Chain, db: &Arc<dyn Db>, known_block: Block) -> Result<u64> {
    let mut block = known_block;

    info!(
        "fast-forwarding chain {} from block {}",
        chain, block.block_number
    );

    let next_block_number_to_sync = loop {
        let prev_block_number = block.prev_block_number.expect("not genesis block");

        let prev_block = load_block(chain, db, prev_block_number).await?;

        if let Some(prev_block) = prev_block {
            if prev_block.hash != block.parent_hash {
                break prev_block_number;
            } else {
                block = prev_block;
            }
        } else {
            break prev_block_number;
        }
    };

    info!(
        "fast-forwarded chain {} to block {}",
        chain, next_block_number_to_sync
    );

    Ok(next_block_number_to_sync)
}

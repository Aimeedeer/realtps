use crate::client::Client;
use crate::delay;
use anyhow::Result;
use log::{debug, info, warn};
use realtps_common::{Chain, Db};
use std::sync::Arc;
use tokio::task;

pub async fn import(chain: Chain, client: &dyn Client, db: &Arc<dyn Db>) -> Result<()> {
    info!("beginning import for {}", chain);

    let head_block_number = client.get_block_number().await?;
    let head_block_number = head_block_number;
    debug!("head block number for {}: {}", chain, head_block_number);

    let highest_block_number = db.load_highest_block_number(chain)?;

    if let Some(highest_block_number) = highest_block_number {
        debug!(
            "highest block number for {}: {}",
            chain, highest_block_number
        );
        if head_block_number < highest_block_number {
            warn!(
                "head_block_number < highest_block_number for chain {}. head: {}; highest: {}",
                chain, head_block_number, highest_block_number
            )
        } else {
            let needed_blocks = head_block_number - highest_block_number;
            info!("importing {} blocks for {}", needed_blocks, chain);
        }
    } else {
        info!("no highest block number for {}", chain);
    }

    if Some(head_block_number) != highest_block_number {
        let initial_sync = highest_block_number.is_none();
        const INITIAL_SYNC_BLOCKS: u64 = 5; // Probably enough to avoid equal or non-monotonic timestamps
        let mut synced = 0;

        let mut block_number = head_block_number;

        loop {
            debug!("fetching block {} for {}", block_number, chain);

            let block = loop {
                let block = client.get_block(block_number).await?;

                if let Some(block) = block {
                    break block;
                } else {
                    debug!(
                        "received no block for number {} on chain {}",
                        block_number, chain
                    );
                    delay::retry_delay().await;
                }
            };

            let parent_hash = block.parent_hash.clone();

            let prev_block_number = block.prev_block_number;
            {
                let db = db.clone();
                task::spawn_blocking(move || db.store_block(block)).await??;
            }

            synced += 1;

            if initial_sync && synced == INITIAL_SYNC_BLOCKS {
                info!("finished initial sync for {}", chain);
                break;
            }

            if let Some(prev_block_number) = prev_block_number {
                let db = db.clone();
                let prev_block =
                    task::spawn_blocking(move || db.load_block(chain, prev_block_number)).await??;

                if let Some(prev_block) = prev_block {
                    if prev_block.hash != parent_hash {
                        warn!(
                            "reorg of chain {} at block {}; old hash: {}; new hash: {}",
                            chain, prev_block_number, prev_block.hash, parent_hash
                        );
                        // continue - have wrong version of prev block
                    } else {
                        if let Some(highest_block_number) = highest_block_number {
                            if prev_block_number <= highest_block_number {
                                info!(
                                    "completed import of chain {} to block {} / {}",
                                    chain, prev_block_number, parent_hash
                                );
                                break;
                            } else {
                                warn!(
                                    "found incomplete previous import for {} at block {}",
                                    chain, prev_block_number
                                );
                                // Found a run of blocks from a previous incomplete import.
                                // Keep going and overwrite them.
                                // continue
                            }
                        } else {
                            warn!(
                                "found incomplete previous import for {} at block {}",
                                chain, prev_block_number
                            );
                            // Found a run of blocks from a previous incomplete import.
                            // Keep going and overwrite them.
                            // continue
                        }
                    }
                } else {
                    // continue - don't have the prev block
                }

                debug!("still need block {} for {}", prev_block_number, chain);
                block_number = prev_block_number;

                delay::courtesy_delay(chain).await;

                continue;
            } else {
                info!("completed import of chain {} to genesis", chain);
                break;
            }
        }

        {
            let db = db.clone();
            task::spawn_blocking(move || db.store_highest_block_number(chain, head_block_number))
                .await??;
        }
    } else {
        info!("no new blocks for {}", chain);
    }

    delay::rescan_delay(chain).await;

    Ok(())
}

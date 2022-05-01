use crate::client::Client;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{debug, trace};
use realtps_common::{chain::Chain, db::Block};
use solana_client::rpc_client::RpcClient;
use solana_transaction_status::UiTransactionEncoding;
use std::sync::Arc;
use tokio::task;

pub struct SolanaClient {
    client: Arc<RpcClient>,
}

impl SolanaClient {
    pub fn new(url: &str) -> Result<Self> {
        let client = Arc::new(RpcClient::new(url.to_string()));

        Ok(SolanaClient { client })
    }
}

#[async_trait]
impl Client for SolanaClient {
    async fn client_version(&self) -> Result<String> {
        let client = self.client.clone();
        let version = task::spawn_blocking(move || client.get_version()).await??;

        Ok(version.solana_core)
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let client = self.client.clone();
        let slot = task::spawn_blocking(move || client.get_slot()).await??;

        Ok(slot)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        // todo: error handling with return missing block
        // `ClientResult<EncodedConfirmedBlock>`

        let client = self.client.clone();
        let block = task::spawn_blocking(move || {
            client.get_block_with_encoding(block_number, UiTransactionEncoding::Base64)
        })
        .await??;

        solana_block_to_block(block, block_number).map(Some)
    }
}

fn solana_block_to_block(
    block: solana_transaction_status::EncodedConfirmedBlock,
    slot_number: u64,
) -> Result<Block> {
    fn calc_user_txs(block: &solana_transaction_status::EncodedConfirmedBlock) -> u64 {
        let mut num_user_txs = 0;
        for tx_status in &block.transactions {
            let tx = tx_status.transaction.decode().unwrap();
            trace!("tx_meta: {:#?}", tx_status.meta.as_ref().unwrap());
            trace!("tx: {:#?}", tx);
            let account_keys = &tx.message.account_keys;
            let mut num_vote_instrs = 0;
            for instr in &tx.message.instructions {
                let program_id_index = instr.program_id_index;
                let program_id = account_keys[usize::from(program_id_index)];

                if program_id == solana_sdk::vote::program::id() {
                    num_vote_instrs += 1;
                    trace!("found vote instruction");
                } else {
                    trace!("non-vote instruction");
                }
            }
            if num_vote_instrs == tx.message.instructions.len() {
                trace!("it's a vote transaction");
            } else {
                // This doesn't look like a vote transaction
                trace!("it's a non-vote transaction");
                num_user_txs += 1;
            }
        }

        let vote_txs = block
            .transactions
            .len()
            .checked_sub(num_user_txs)
            .expect("underflow");
        debug!("solana total txs: {}", block.transactions.len());
        debug!("solana user txs: {}", num_user_txs);
        debug!("solana vote txs: {}", vote_txs);

        u64::try_from(num_user_txs).expect("u64")
    }

    Ok(Block {
        chain: Chain::Solana,
        block_number: slot_number,
        prev_block_number: Some(block.parent_slot),
        timestamp: u64::try_from(
            block
                .block_time
                .ok_or_else(|| anyhow!("block time unavailable for solana slot {}", slot_number))?,
        )?,
        num_txs: calc_user_txs(&block),
        hash: block.blockhash,
        parent_hash: block.previous_blockhash,
    })
}

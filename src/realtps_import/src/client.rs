use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ethers::prelude::*;
use ethers::utils::hex::ToHex;
use near_jsonrpc_client::{auth::Unauthenticated, methods, JsonRpcClient};
use near_jsonrpc_primitives::types::chunks::ChunkReference;
use near_primitives::{
    types::{BlockId, BlockReference},
    views::{BlockHeaderView, BlockView, ChunkHeaderView},
};
use realtps_common::{Block, Chain};
use solana_client::rpc_client::RpcClient;
use solana_transaction_status::UiTransactionEncoding;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use log::{debug, trace};

#[async_trait]
pub trait Client: Send + Sync + 'static {
    async fn client_version(&self) -> Result<String>;
    async fn get_latest_block_number(&self) -> Result<u64>;
    async fn get_block(&self, block_number: u64) -> Result<Option<Block>>;
}

pub struct EthersClient {
    pub chain: Chain,
    pub provider: Provider<Http>,
}

impl EthersClient {
    pub fn new(chain: Chain, url: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(url)?;

        Ok(EthersClient { chain, provider })
    }
}

#[async_trait]
impl Client for EthersClient {
    async fn client_version(&self) -> Result<String> {
        Ok(self.provider.client_version().await?)
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        Ok(self.provider.get_block_number().await?.as_u64())
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        if let Some(block) = self.provider.get_block(block_number).await? {
            // I like this `map` <3
            ethers_block_to_block(self.chain, block).map(Some)
        } else {
            Ok(None)
        }
    }
}

pub struct NearClient {
    pub client: JsonRpcClient<Unauthenticated>,
}

impl NearClient {
    pub fn new(url: &str) -> Result<Self> {
        let client = JsonRpcClient::connect(url);

        Ok(NearClient { client })
    }
}

#[async_trait]
impl Client for NearClient {
    async fn client_version(&self) -> Result<String> {
        let client = self.client.clone();
        let status = client.call(methods::status::RpcStatusRequest).await?;

        Ok(status.version.version)
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let client = self.client.clone();
        let status = client.call(methods::status::RpcStatusRequest).await?;

        Ok(status.sync_info.latest_block_height)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let client = self.client.clone();
        let block = client
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::BlockId(BlockId::Height(block_number)),
            })
            .await?;

        // caculating total tx numbers from chunks in the block
        let mut num_txs: usize = 0;
        for chunk_head in &block.chunks {
            let client = self.client.clone();

            let chunk = client
                .call(methods::chunk::RpcChunkRequest {
                    chunk_reference: ChunkReference::ChunkHash {
                        chunk_id: chunk_head.chunk_hash,
                    },
                })
                .await?;

            let txs = chunk.transactions.len();
            num_txs = num_txs.checked_add(txs).expect("number of txs overflow");
        }

        let num_txs = u64::try_from(num_txs)?;
        near_block_to_block(block, block_number, num_txs).map(Some)
    }
}

pub struct SolanaClient {
    pub client: Arc<RpcClient>,
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
        let version = task::spawn_blocking(move || client.get_version());

        Ok(version.await??.solana_core)
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let client = self.client.clone();
        let slot = task::spawn_blocking(move || client.get_slot());

        Ok(slot.await??)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        // todo: error handling with return missing block
        // `ClientResult<EncodedConfirmedBlock>`

        let client = self.client.clone();
        let block = task::spawn_blocking(move || {
            client.get_block_with_encoding(block_number, UiTransactionEncoding::Base64)
        });

        solana_block_to_block(block.await??, block_number).map(Some)
    }
}

fn ethers_block_to_block(chain: Chain, block: ethers::prelude::Block<H256>) -> Result<Block> {
    let block_number = block.number.expect("block number").as_u64();
    Ok(Block {
        chain,
        block_number: block_number,
        prev_block_number: block_number.checked_sub(1),
        timestamp: u64::try_from(block.timestamp).map_err(|e| anyhow!("{}", e))?,
        num_txs: u64::try_from(block.transactions.len())?,
        hash: block.hash.expect("hash").encode_hex(),
        parent_hash: block.parent_hash.encode_hex(),
    })
}

fn near_block_to_block(block: BlockView, block_number: u64, num_txs: u64) -> Result<Block> {
    Ok(Block {
        chain: Chain::Near,
        block_number,
        prev_block_number: block.header.prev_height,
        timestamp: Duration::from_nanos(block.header.timestamp_nanosec).as_secs(),
        num_txs,
        hash: block.header.hash.to_string(),
        parent_hash: block.header.prev_hash.to_string(),
    })
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

        let vote_txs = block.transactions.len().checked_sub(num_user_txs).expect("underflow");
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

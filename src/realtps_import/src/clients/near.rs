use crate::client::Client;
use anyhow::Result;
use async_trait::async_trait;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::chunks::ChunkReference;
use near_primitives::{
    types::{BlockId, BlockReference},
    views::BlockView,
};
use realtps_common::{chain::Chain, db::Block};
use std::time::Duration;

pub struct NearClient {
    client: JsonRpcClient,
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
        let status = self.client.call(methods::status::RpcStatusRequest).await?;

        Ok(status.version.version)
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let status = self.client.call(methods::status::RpcStatusRequest).await?;

        Ok(status.sync_info.latest_block_height)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let block = self
            .client
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::BlockId(BlockId::Height(block_number)),
            })
            .await?;

        // caculating total tx numbers from chunks in the block
        let mut num_txs: usize = 0;
        for chunk_head in &block.chunks {
            let chunk = self
                .client
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

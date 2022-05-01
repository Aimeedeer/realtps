use anyhow::Result;
use async_trait::async_trait;
use realtps_common::db::Block;

#[async_trait]
pub trait Client: Send + Sync + 'static {
    async fn client_version(&self) -> Result<String>;
    async fn get_latest_block_number(&self) -> Result<u64>;
    /// Returns `None` if the network thinks the block doesn't exist
    async fn get_block(&self, block_number: u64) -> Result<Option<Block>>;
}

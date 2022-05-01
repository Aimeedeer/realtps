use crate::client::Client;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ethers::prelude::*;
use ethers::utils::hex::ToHex;
use realtps_common::{chain::Chain, db::Block};

pub struct EthersClient {
    chain: Chain,
    provider: Provider<Http>,
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

fn ethers_block_to_block(chain: Chain, block: ethers::prelude::Block<H256>) -> Result<Block> {
    let block_number = block.number.expect("block number").as_u64();
    Ok(Block {
        chain,
        block_number,
        prev_block_number: block_number.checked_sub(1),
        timestamp: u64::try_from(block.timestamp).map_err(|e| anyhow!("{}", e))?,
        num_txs: u64::try_from(block.transactions.len())?,
        hash: block.hash.expect("hash").encode_hex(),
        parent_hash: block.parent_hash.encode_hex(),
    })
}

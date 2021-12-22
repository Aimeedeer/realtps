use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ethers::prelude::*;
use ethers::utils::hex::ToHex;
use realtps_common::{Block, Chain};
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use tokio::task;
use solana_transaction_status::UiTransactionEncoding;

#[async_trait]
pub trait Client: Send + Sync + 'static {
    async fn client_version(&self) -> Result<String>;
    async fn get_block_number(&self) -> Result<u64>;
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

    async fn get_block_number(&self) -> Result<u64> {
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

    async fn get_block_number(&self) -> Result<u64> {
        let client = self.client.clone();
        let slot = task::spawn_blocking(move || client.get_slot());

        Ok(slot.await??)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        // todo: error handling with return missing block
        // `ClientResult<EncodedConfirmedBlock>`

        let client = self.client.clone();
        let block = task::spawn_blocking(move || client.get_block_with_encoding(block_number, UiTransactionEncoding::Base64));

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

fn solana_block_to_block(
    block: solana_transaction_status::EncodedConfirmedBlock,
    slot_number: u64,
) -> Result<Block> {
    Ok(Block {
        chain: Chain::Solana,
        block_number: slot_number,
        prev_block_number: Some(block.parent_slot),
        timestamp: u64::try_from(
            block
                .block_time
                .ok_or_else(|| anyhow!("block time unavailable for solana slot {}", slot_number))?,
        )?,
        num_txs: u64::try_from(block.transactions.len())?,
        hash: block.blockhash,
        parent_hash: block.previous_blockhash,
    })
}

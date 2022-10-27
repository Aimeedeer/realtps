use crate::client::Client;
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use realtps_common::{chain::Chain, db::Block};
use serde::Deserialize;
use std::str::FromStr;

pub struct ElectrumClient {
    url: String,
}

#[derive(Deserialize, Debug)]
struct ElectrumBlock {
    id: String,
    height: u64,
    version: u32,
    timestamp: u32,
    tx_count: u32,
    previousblockhash: String,
    nonce: u32,
}

impl ElectrumClient {
    pub fn new(url: &str) -> Result<Self> {
        Ok(ElectrumClient {
            url: url.to_string(),
        })
    }
}

#[async_trait]
impl Client for ElectrumClient {
    async fn client_version(&self) -> Result<String> {
        let block_hash = reqwest::get(format!("{}{}", self.url, "blocks/tip/hash"))
            .await?
            .text()
            .await?;
        let block: ElectrumBlock = reqwest::get(format!("{}{}/{}", self.url, "block", block_hash))
            .await?
            .json()
            .await?;

        Ok(block.version.to_string())
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let block_number = reqwest::get(format!("{}{}", self.url, "blocks/tip/height"))
            .await?
            .text()
            .await?;

        Ok(u64::from_str(&block_number).expect("u64"))
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let block_hash = reqwest::get(format!("{}{}/{}", self.url, "block-height", block_number))
            .await?
            .text()
            .await?;
        let block: ElectrumBlock = reqwest::get(format!("{}{}/{}", self.url, "block", block_hash))
            .await?
            .json()
            .await?;

        let prev_block: ElectrumBlock = reqwest::get(format!(
            "{}{}/{}",
            self.url, "block", block.previousblockhash
        ))
        .await?
        .json()
        .await?;

        let block = Block {
            chain: Chain::Bitcoin,
            block_number,
            prev_block_number: Some(prev_block.height),
            timestamp: u64::from(block.timestamp),
            num_txs: u64::from(block.tx_count),
            hash: block.id,
            parent_hash: block.previousblockhash,
        };

        Ok(Some(block))
    }
}

use crate::client::Client;
use anyhow::Result;
use async_trait::async_trait;
use realtps_common::{chain::Chain, db::Block};

pub struct HederaClient {
    client: reqwest::Client,
    url: String,
}

impl HederaClient {
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
        })
    }

    async fn get_most_recent_block(&self) -> Result<HederaBlockResponse> {
        let url = format!("{}/api/v1/blocks?order=desc&limit=1", self.url);
        let resp = self.client.get(url).send().await?;
        let recent_blocks: HederaBlockResponse = resp.json().await?;
        Ok(recent_blocks)
    }
}

#[derive(serde::Deserialize)]
struct HederaBlockResponse {
    blocks: Vec<HederaBlock>,
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
struct HederaBlock {
    count: i64,
    hapi_version: String,
    hash: String,
    name: String,
    number: i64,
    previous_hash: String,
    size: i64,
    timestamp: Timestamp,
    gas_used: i64,
    logs_bloom: String,
}

#[derive(serde::Deserialize, Debug)]
struct Timestamp {
    to: String,
}

#[async_trait]
impl Client for HederaClient {
    // Hedera mirror node doesn't report the version number
    async fn client_version(&self) -> Result<String> {
        let recent_blocks = self.get_most_recent_block().await?;
        Ok(recent_blocks.blocks[0].hapi_version.clone())
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let recent_blocks = self.get_most_recent_block().await?;
        Ok(recent_blocks.blocks[0].number as u64)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let url = format!("{}/api/v1/blocks/{}", &self.url, block_number);
        let response = self.client.get(url).send().await?;
        let block: HederaBlock = response.json().await?;

        Ok(Some(Block {
            chain: Chain::Hedera,
            block_number,
            prev_block_number: if block_number > 0 {
                Some(block_number - 1)
            } else {
                None
            },
            timestamp: block.timestamp.to.parse::<f64>().unwrap() as u64,
            num_txs: block.count as u64,
            hash: block.hash,
            parent_hash: block.previous_hash,
        }))
    }
}

use crate::client::Client;
use anyhow::Result;
use async_trait::async_trait;
use realtps_common::{chain::Chain, db::Block};

pub struct PivxClient {
    client: reqwest::Client,
    url: String,
}

impl PivxClient {
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
        })
    }
    pub async fn get_block_hash(&self, block_number: u64) -> Result<String> {
        let url = format!(
            "{}/pivx/api.dws?q=getblockhash&height={}",
            &self.url, block_number
        );
        let resp = self.client.get(url).send().await?;
        let hash: String = resp.json().await?;
        return Ok(hash);
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PivxBlockInfo {
    hash: String,
    previousblockhash: String,
    tx: Vec<String>,
    time: u64,
}

#[async_trait]
impl Client for PivxClient {
    async fn client_version(&self) -> Result<String> {
        Ok("https://chainz.cryptoid.info".to_string()) // hardcoded since version is not available in API
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let url = format!("{}/pivx/api.dws?q=getblockcount", &self.url);
        let resp = self.client.get(url).send().await?;
        let block_number: u64 = resp.json().await?;
        Ok(block_number)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        // use block hash to get full details
        let hash = self.get_block_hash(block_number).await?;

        let url = format!(
            "{}/explorer/block.raw.dws?coin=pivx&hash={}.js",
            &self.url, hash
        );
        let resp = self.client.get(url).send().await?;
        let block_info: PivxBlockInfo = resp.json().await?;

        Ok(Some(Block {
            chain: Chain::Pivx,
            block_number,
            prev_block_number: if block_number > 0 {
                Some(block_number - 1)
            } else {
                None
            },
            timestamp: block_info.time,
            num_txs: block_info.tx.len() as u64,
            hash: block_info.hash,
            parent_hash: block_info.previousblockhash,
        }))
    }
}

#[cfg(test)]
mod test_pivx {
    use super::{Client, PivxClient};

    // Block count: https://chainz.cryptoid.info/pivx/api.dws?q=getblockcount
    // Block Hash; https://chainz.cryptoid.info/pivx/api.dws?q=getblockhash&height=3598398
    // Block 1: https://chainz.cryptoid.info/explorer/block.raw.dws?coin=pivx&hash=000005504fa4a6766e854b2a2c3f21cd276fd7305b84f416241fd4431acbd12d.js
    const API_URL: &str = "https://chainz.cryptoid.info";

    #[tokio::test]
    async fn client_version() -> Result<(), anyhow::Error> {
        let client = PivxClient::new(API_URL)?;
        let ver = client.client_version().await?;
        println!("PIVX client_version: {}", ver);
        assert!(!ver.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn get_latest_block_number() -> Result<(), anyhow::Error> {
        let client = PivxClient::new(API_URL)?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("PIVX latest_block_number: {}", latest_block_number);
        assert!(latest_block_number > 0);
        Ok(())
    }

    #[tokio::test]
    async fn get_block() -> Result<(), anyhow::Error> {
        let client = PivxClient::new(API_URL)?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("PIVX latest_block_number: {}", latest_block_number);
        let block = client.get_block(latest_block_number).await?;
        println!("PIVX block: {:?}", block);
        Ok(())
    }
}

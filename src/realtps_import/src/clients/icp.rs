use crate::client::Client;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use realtps_common::{chain::Chain, db::Block};

pub struct IcpClient {
    client: reqwest::Client,
    url: String,
}

impl IcpClient {
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
        })
    }
}

#[derive(serde::Deserialize, Debug)]
struct IcpNetworkOptionsResponse {
    version: serde_json::Value,
}

#[derive(serde::Deserialize, Debug)]
struct IcpNetworkStatusResponse {
    current_block_identifier: serde_json::Value,
}

#[derive(serde::Deserialize, Debug)]
struct IcpBlockResponse {
    block: serde_json::Value,
}

#[async_trait]
impl Client for IcpClient {
    async fn client_version(&self) -> Result<String> {
        let body = serde_json::json!({
            "network_identifier": {
                "blockchain": "Internet Computer",
                "network": "00000000000000020101"
            }
        });
        let url = format!("{}/network/options", &self.url);
        let resp = self.client.post(&url).json(&body).send().await?;
        let network_options: IcpNetworkOptionsResponse = resp.json().await?;
        Ok(network_options
            .version
            .get("rosetta_version")
            .ok_or_else(|| anyhow!("no rosetta_version key"))?
            .as_str()
            .ok_or_else(|| anyhow!("not a string"))?
            .to_string())
    }
    async fn get_latest_block_number(&self) -> Result<u64> {
        let body = serde_json::json!({
            "network_identifier": {
                "blockchain": "Internet Computer",
                "network": "00000000000000020101"
            }
        });
        let url = format!("{}/network/status", &self.url);
        let resp = self.client.post(&url).json(&body).send().await?;
        let network_status: IcpNetworkStatusResponse = resp.json().await?;
        Ok(network_status
            .current_block_identifier
            .get("index")
            .ok_or_else(|| anyhow!("no index key"))?
            .as_u64()
            .ok_or_else(|| anyhow!("not a u64"))?)
    }
    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let body = serde_json::json!({
            "network_identifier": {
                "blockchain": "Internet Computer",
                "network": "00000000000000020101"
            },
            "block_identifier": {
                "index": block_number
            }
        });
        let url = format!("{}/block", &self.url);
        let resp = self.client.post(&url).json(&body).send().await?;
        let response: IcpBlockResponse = resp.json().await?;
        let block = response.block;

        let block_identifier = block
            .get("block_identifier")
            .ok_or_else(|| anyhow!("no block_identifier key"))?;
        let parent_block_identifier = block
            .get("parent_block_identifier")
            .ok_or_else(|| anyhow!("no parent_block_identifier key"))?;
        let block_number = block_identifier
            .get("index")
            .ok_or_else(|| anyhow!("no index key"))?
            .as_u64()
            .ok_or_else(|| anyhow!("not a u64"))?;
        let prev_block_number = parent_block_identifier
            .get("index")
            .ok_or_else(|| anyhow!("no index key"))?
            .as_u64();
        let hash = block_identifier
            .get("hash")
            .ok_or_else(|| anyhow!("no hash key"))?
            .as_str()
            .ok_or_else(|| anyhow!("not a string"))?
            .to_string();
        let parent_hash = parent_block_identifier
            .get("hash")
            .ok_or_else(|| anyhow!("no hash key"))?
            .as_str()
            .ok_or_else(|| anyhow!("not a string"))?
            .to_string();
        let timestamp = block
            .get("timestamp")
            .ok_or_else(|| anyhow!("no timestamp key"))?
            .as_u64()
            .ok_or_else(|| anyhow!("not a u64"))?
            / 1000;
        let num_txs = block
            .get("transactions")
            .ok_or_else(|| anyhow!("no transactions key"))?
            .as_array()
            .ok_or_else(|| anyhow!("not an array"))?
            .len() as u64;

        Ok(Some(Block {
            chain: Chain::InternetComputer,
            block_number,
            prev_block_number,
            timestamp,
            num_txs,
            hash,
            parent_hash,
        }))
    }
}

#[cfg(test)]
mod test_icp {
    use super::{Client, IcpClient};

    const URL: &str = "https://rosetta-api.internetcomputer.org";

    #[tokio::test]
    async fn client_version() -> Result<(), anyhow::Error> {
        let client = IcpClient::new(URL)?;
        let ver = client.client_version().await?;
        println!("client_version: {}", ver);
        assert!(ver.len() > 0);
        Ok(())
    }

    #[tokio::test]
    async fn get_latest_block_number() -> Result<(), anyhow::Error> {
        let client = IcpClient::new(URL)?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("latest_block_number: {}", latest_block_number);
        assert!(latest_block_number > 0);
        Ok(())
    }

    #[tokio::test]
    async fn get_block() -> Result<(), anyhow::Error> {
        let client = IcpClient::new(URL)?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("latest_block_number: {}", latest_block_number);
        let block = client.get_block(latest_block_number).await?;
        println!("block: {:?}", block);
        Ok(())
    }
}

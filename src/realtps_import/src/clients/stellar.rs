use crate::client::Client;
use anyhow::Result;
use async_trait::async_trait;
use realtps_common::{chain::Chain, db::Block};

pub struct StellarClient {
    client: reqwest::Client,
    url: String,
}

impl StellarClient {
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
        })
    }
}

#[derive(serde::Deserialize)]
struct StellarNetworkDetailsResponse {
    horizon_version: String,
    history_latest_ledger: u32,
}

#[derive(serde::Deserialize)]
struct StellarLedgerResponse {
    hash: String,
    prev_hash: String,
    closed_at: chrono::DateTime<chrono::Utc>,
    successful_transaction_count: u32,
    failed_transaction_count: u32,
    operation_count: u32,
}

#[async_trait]
impl Client for StellarClient {
    async fn client_version(&self) -> Result<String> {
        let resp = self.client.get(&self.url).send().await?;
        let network_details: StellarNetworkDetailsResponse = resp.json().await?;
        Ok(network_details.horizon_version)
    }
    async fn get_latest_block_number(&self) -> Result<u64> {
        let resp = self.client.get(&self.url).send().await?;
        let network_details: StellarNetworkDetailsResponse = resp.json().await?;
        Ok(network_details.history_latest_ledger as u64)
    }
    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let url = format!("{}/ledgers/{}", &self.url, block_number);
        let resp = self.client.get(url).send().await?;
        let ledger: StellarLedgerResponse = resp.json().await?;

        let num_txs = (ledger.successful_transaction_count as u64)
            .checked_add(ledger.failed_transaction_count as u64)
            .expect("overflow");

        Ok(Some(Block {
            chain: Chain::Stellar,
            block_number,
            prev_block_number: if block_number > 0 {
                Some(block_number - 1)
            } else {
                None
            },
            timestamp: ledger.closed_at.timestamp() as u64,
            num_txs,
            hash: ledger.hash,
            parent_hash: ledger.prev_hash,
        }))
    }
}

#[cfg(test)]
mod test_stellar {
    use super::{Client, StellarClient};

    const RPC_URL: &str = "https://horizon.stellar.org";

    #[tokio::test]
    async fn client_version() -> Result<(), anyhow::Error> {
        let client = StellarClient::new(RPC_URL)?;
        let ver = client.client_version().await?;
        println!("client_version: {}", ver);
        assert!(!ver.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn get_latest_block_number() -> Result<(), anyhow::Error> {
        let client = StellarClient::new(RPC_URL)?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("latest_block_number: {}", latest_block_number);
        assert!(latest_block_number > 0);
        Ok(())
    }

    #[tokio::test]
    async fn get_block() -> Result<(), anyhow::Error> {
        let client = StellarClient::new(RPC_URL)?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("latest_block_number: {}", latest_block_number);
        let block = client.get_block(latest_block_number).await?;
        println!("block: {:?}", block);
        Ok(())
    }
}

use crate::client::Client;
use algonaut::{algod::v2::Algod, indexer::v2::Indexer};
use anyhow::Result;
use async_trait::async_trait;
use hex::ToHex;
use realtps_common::{chain::Chain, db::Block};

pub struct AlgorandClient {
    algod: Algod,
    indexer: Indexer,
}

impl AlgorandClient {
    pub fn new(urls: &str) -> Result<Self> {
        let urls: Vec<&str> = urls.split(';').collect();
        let algod_url = urls.get(0).expect("algorand algod url");
        let indexer_url = urls.get(1).expect("algorand indexer url");
        Ok(Self {
            algod: Algod::with_headers(algod_url, vec![])?,
            indexer: Indexer::with_headers(indexer_url, vec![])?,
        })
    }
}

#[async_trait]
impl Client for AlgorandClient {
    async fn client_version(&self) -> Result<String> {
        let versions = self.algod.versions().await?;
        Ok(versions.build.semver())
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let status = self.algod.status().await?;
        Ok(status.last_round)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let block = self
            .algod
            .block_with_certificate(block_number.into())
            .await?;

        let indexer_block = self.indexer.block(block_number.into()).await?;

        Ok(Some(Block {
            chain: Chain::Algorand,
            block_number,
            prev_block_number: if block_number > 0 {
                Some(block_number - 1)
            } else {
                None
            },
            timestamp: indexer_block.timestamp,
            num_txs: indexer_block.transactions.len() as u64,
            hash: block.hash().encode_hex(),
            parent_hash: indexer_block.previous_block_hash.encode_hex(),
        }))
    }
}

#[cfg(test)]
mod test_algorand {
    use anyhow::Result;

    use super::{AlgorandClient, Client};

    fn create_client() -> Result<AlgorandClient> {
        AlgorandClient::new()
    }

    #[tokio::test]
    async fn client_version() -> Result<(), anyhow::Error> {
        let client = create_client()?;
        let version = client.client_version().await?;
        println!("version: {version}");
        assert!(!version.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn get_latest_block_number() -> Result<(), anyhow::Error> {
        let client = create_client()?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("latest_block_number: {latest_block_number}");
        assert!(latest_block_number > 0);
        Ok(())
    }

    #[tokio::test]
    async fn get_block() -> Result<(), anyhow::Error> {
        let client = create_client()?;
        let latest_block_number = client.get_latest_block_number().await?;
        let block = client.get_block(latest_block_number).await?;
        println!("block: {block:?}");
        Ok(())
    }
}

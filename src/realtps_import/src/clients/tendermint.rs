use crate::client::Client;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use realtps_common::{chain::Chain, db::Block};
use tendermint_rpc::{Client as TendermintClientTrait, HttpClient};

pub struct TendermintClient {
    chain: Chain,
    client: HttpClient,
}

impl TendermintClient {
    pub fn new(chain: Chain, url: &str) -> Result<Self> {
        let client = HttpClient::new(url)?;

        Ok(TendermintClient { chain, client })
    }
}

#[async_trait]
impl Client for TendermintClient {
    async fn client_version(&self) -> Result<String> {
        let status = self.client.status().await?;

        Ok(status.node_info.moniker.to_string())
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let status = self.client.status().await?;

        Ok(status.sync_info.latest_block_height.value())
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let tendermint_block_height = tendermint::block::Height::try_from(block_number)?;
        let block_response = self.client.block(tendermint_block_height).await?;

        tendermint_block_to_block(self.chain, block_response, block_number).map(Some)
    }
}

fn tendermint_block_to_block(
    chain: Chain,
    block_response: tendermint_rpc::endpoint::block::Response,
    block_number: u64,
) -> Result<Block> {
    Ok(Block {
        chain,
        block_number,
        prev_block_number: block_number.checked_sub(1),
        timestamp: u64::try_from(
            tendermint_proto::google::protobuf::Timestamp::from(block_response.block.header.time)
                .seconds,
        )?,
        num_txs: u64::try_from(block_response.block.data.iter().count())?,
        hash: block_response.block_id.hash.to_string(),
        parent_hash: block_response
            .block
            .header
            .last_block_id
            .ok_or_else(|| anyhow!("no previous block id"))?
            .hash
            .to_string(),
    })
}

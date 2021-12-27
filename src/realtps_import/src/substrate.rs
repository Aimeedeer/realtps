use crate::client::Client;
use anyhow::Result;
use async_trait::async_trait;
use realtps_common::{chain::Chain, db::Block};
use subxt::sp_runtime::generic::Block as SubstrateBlock;
use subxt::sp_runtime::traits::{Block as SubstrateBlockTrait, Header};
use subxt::{BlockNumber, ClientBuilder};

#[subxt::subxt(runtime_metadata_path = "substrate_meta/polkadot.scale")]
pub mod polkadot {}

pub struct PolkadotClient {
    client: subxt::Client<polkadot::DefaultConfig>,
}

impl PolkadotClient {
    pub async fn new(url: &str) -> Result<Self> {
        let client = ClientBuilder::new().set_url(url).build().await?;
        Ok(PolkadotClient { client })
    }
}

#[async_trait]
impl Client for PolkadotClient {
    async fn client_version(&self) -> Result<String> {
        let rpc = self.client.rpc();
        let version = rpc.runtime_version(None).await?;
        Ok(version.spec_name.to_string())
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let rpc = self.client.rpc();
        let header = rpc.header(None).await?.expect("header");
        let number = u64::from(*header.number());
        Ok(number)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let rpc = self.client.rpc();
        let block_number = u32::try_from(block_number)?;
        let block_number = BlockNumber::from(block_number);
        let hash = rpc.block_hash(Some(block_number)).await?;

        if hash.is_none() {
            return Ok(None);
        }

        let hash = hash.unwrap();

        let block = rpc.block(Some(hash)).await?;

        if block.is_none() {
            return Ok(None);
        }

        let block = block.unwrap();
        let block = block.block;

        let runtime = self
            .client
            .clone()
            .to_runtime_api::<polkadot::RuntimeApi<polkadot::DefaultConfig>>();
        let timestamp = runtime.storage().timestamp().now(Some(hash)).await?;

        polkadot_block_to_block(block, timestamp).map(Some)
    }
}

type PolkadotBlock = SubstrateBlock<
    <polkadot::DefaultConfig as subxt::Config>::Header,
    <polkadot::DefaultConfig as subxt::Config>::Extrinsic,
>;

fn polkadot_block_to_block(block: PolkadotBlock, timestamp: u64) -> Result<Block> {
    let chain = Chain::Polkadot;
    let block_number = u64::from(*block.header.number());
    let prev_block_number = block_number.checked_sub(1);
    let timestamp = timestamp;
    let num_txs = u64::try_from(block.extrinsics().len()).expect("u64");
    let hash = block.hash().to_string();
    let parent_hash = block.header().parent_hash().to_string();

    Ok(Block {
        chain,
        block_number,
        prev_block_number,
        timestamp,
        num_txs,
        hash,
        parent_hash,
    })
}

#![allow(unused)]

use anyhow::Result;
use crate::client::Client;
use realtps_common::{chain::Chain, db::Block};
use async_trait::async_trait;
use substrate_api_client::{Api, Metadata};
use sp_core::sr25519;
use sp_runtime::generic::{Block as SubstrateBlock};
use sp_runtime::traits::BlakeTwo256;
use tokio::task;

// Copied from polkadot-core-primitives
pub type BlockNumber = u32;
pub type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
// Copied from polkadot-runtime
//pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

pub struct PolkadotClient {
}

impl PolkadotClient {
    pub async fn new(url: &str) -> Result<Self> {
        todo!()
    }
}

#[async_trait]
impl Client for PolkadotClient {
    async fn client_version(&self) -> Result<String> {
        todo!()
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        todo!()
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        todo!()
    }
}

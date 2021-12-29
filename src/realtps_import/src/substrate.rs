#![allow(unused)]

use anyhow::Result;
use crate::client::Client;
use realtps_common::{chain::Chain, db::Block};
use async_trait::async_trait;
use substrate_api_client::rpc::WsRpcClient;
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
    api: Api<sr25519::Pair, WsRpcClient>,
}

impl PolkadotClient {
    pub async fn new(url: &str) -> Result<Self> {
        let client = WsRpcClient::new(url);
        let api = task::spawn_blocking(move || Api::new(client)).await??;

        Ok(PolkadotClient {
            api
        })
    }
}

#[async_trait]
impl Client for PolkadotClient {
    async fn client_version(&self) -> Result<String> {
        Ok(format!("{}", self.api.runtime_version))
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let api = self.api.clone();
        let header = task::spawn_blocking(move || api.get_header(None)).await??;
        let header = header.expect("header");
        let header: Header = header;
        let number = u64::from(header.number);

        Ok(number)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        /*let api = self.api.clone();
        let block_number = u32::try_from(block_number)?;
        let block = task::spawn_blocking(move || api.get_block_by_num(Some(block_number))).await??;

        if let Some(block) = block {
            let block: SubstrateBlock<Header<u32, BlakeTwo256>, CheckedExtrinsic> = block;
            todo!()
        } else {
            Ok(None)
    }*/
        todo!()
    }
}

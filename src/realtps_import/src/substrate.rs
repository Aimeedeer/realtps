#![allow(unused)]

use hex::FromHex;
use log::trace;
use anyhow::{anyhow, Result};
use crate::client::Client;
use realtps_common::{chain::Chain, db::Block};
use async_trait::async_trait;
use tokio::task;
use jsonrpc_core_client::RawClient;
use jsonrpc_core_client::transports::http;
use jsonrpc_core::types::{Params, Value};

pub struct PolkadotClient {
    client: RawClient
}

impl PolkadotClient {
    pub async fn new(url: &str) -> Result<Self> {
        let client = http::connect(url).await
            .map_err(|e| anyhow!("{}", e))?;

        Ok(PolkadotClient {
            client
        })
    }
}

#[async_trait]
impl Client for PolkadotClient {
    async fn client_version(&self) -> Result<String> {
        let runtime_version = self.client.call_method(
            "state_getRuntimeVersion",
            Params::None,
        ).await.map_err(|e| anyhow!("{}", e))?;

        trace!("runtime_version: {:#?}", runtime_version);
        let impl_name = runtime_version.get("implName").expect("implName").as_str().expect("str");

        Ok(impl_name.to_string())
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let header = self.client.call_method(
            "chain_getHeader",
            Params::None,
        ).await.map_err(|e| anyhow!("{}", e))?;

        trace!("header: {:#?}", header);

        let number_hex = header.get("number").expect("number").as_str().expect("str");
        assert!(number_hex.starts_with("0x"));
        let number_hex = number_hex[2..].to_string();
        assert!(number_hex.len() <= 4 * 2);
        let missing_zeros = (4 * 2) - number_hex.len();
        let number_hex = std::iter::repeat('0').take(missing_zeros).chain(number_hex.chars()).collect::<String>();
        let number_bytes = <[u8; 4]>::from_hex(number_hex.as_bytes())?;
        let number = u32::from_be_bytes(number_bytes);
        let number = u64::from(number);

        Ok(number)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let block_number = u32::try_from(block_number)?;
        let hash = self.client.call_method(
            "chain_getBlockHash",
            Params::Array(vec![Value::from(block_number)]),
        ).await.map_err(|e| anyhow!("{}", e))?;

        trace!("hash: {:#?}", hash);

        let hash = hash.as_str().expect("str");

        let block = self.client.call_method(
            "chain_getBlock",
            Params::Array(vec![Value::from(hash)]),
        ).await.map_err(|e| anyhow!("{}", e))?;

        trace!("block: {:#?}", block);

        if block.is_null() {
            return Ok(None);
        }

        let timestamp = todo!();

        substrate_block_to_block(
            Chain::Polkadot,
            block,
            u64::from(block_number),
            hash.to_string(),
            timestamp
        ).map(Some)
    }
}

fn substrate_block_to_block(
    chain: Chain,
    block: Value,
    block_number: u64,
    hash: String,
    timestamp: u64,
) -> Result<Block> {

    let block = block.get("block").expect("block");
    let header = block.get("header").expect("header");
    let extrinsics = block.get("extrinsics").expect("extrinsics").as_array().expect("array");

    let prev_block_number = block_number.checked_sub(1);
    let num_txs = u64::try_from(extrinsics.len())?;
    let parent_hash = header.get("parentHash").expect("parentHash").as_str().expect("str").to_string();

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

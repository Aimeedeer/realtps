use crate::client::Client;
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use hex::FromHex;
use jsonrpc_core::types::{Params, Value};
use jsonrpc_core_client::transports::http;
use jsonrpc_core_client::RawClient;
use log::trace;
use realtps_common::{chain::Chain, db::Block};
use sp_storage::StorageKey;

pub struct SubstrateClient {
    chain: Chain,
    client: RawClient,
}

impl SubstrateClient {
    pub async fn new(chain: Chain, url: &str) -> Result<Self> {
        let client = http::connect(url).await.map_err(|e| anyhow!("{}", e))?;

        Ok(SubstrateClient { chain, client })
    }
}

#[async_trait]
impl Client for SubstrateClient {
    async fn client_version(&self) -> Result<String> {
        let runtime_version = self
            .client
            .call_method("state_getRuntimeVersion", Params::None)
            .await
            .map_err(|e| anyhow!("{}", e))?;

        trace!("runtime_version: {:#?}", runtime_version);
        let impl_name = runtime_version
            .get("implName")
            .expect("implName")
            .as_str()
            .expect("str");

        Ok(impl_name.to_string())
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let header = self
            .client
            .call_method("chain_getHeader", Params::None)
            .await
            .map_err(|e| anyhow!("{}", e))?;

        trace!("header: {:#?}", header);

        let number_hex = header.get("number").expect("number").as_str().expect("str");
        let number = hex_be_to_u32(number_hex)?;
        let number = u64::from(number);

        Ok(number)
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let block_number = u32::try_from(block_number)?;
        let hash = self
            .client
            .call_method(
                "chain_getBlockHash",
                Params::Array(vec![Value::from(block_number)]),
            )
            .await
            .map_err(|e| anyhow!("{}", e))?;

        trace!("hash: {:#?}", hash);

        let hash = match hash.as_str() {
            Some(hash) => hash,
            None => {
                // TODO: figure out why this happens. May need to check for null.
                bail!("substrate hash wasn't a string: {:?}", hash);
            }
        };

        let block = self
            .client
            .call_method("chain_getBlock", Params::Array(vec![Value::from(hash)]))
            .await
            .map_err(|e| anyhow!("{}", e))?;

        trace!("block: {:#?}", block);

        if block.is_null() {
            return Ok(None);
        }

        let timestamp_storage_key = storage_key("Timestamp", "Now");
        let timestamp_storage_key = StorageKey(timestamp_storage_key);
        let timestamp_storage_key = serde_json::to_value(timestamp_storage_key)?;
        let timestamp = self
            .client
            .call_method(
                "state_getStorage",
                Params::Array(vec![timestamp_storage_key, Value::from(hash)]),
            )
            .await
            .map_err(|e| anyhow!("{}", e))?;

        trace!("timestamp: {:#?}", timestamp);

        let timestamp = hex_le_to_u64(timestamp.as_str().expect("str"))?;
        // Timestamp appears to be in ms
        let timestamp = timestamp / 1000;

        substrate_block_to_block(
            self.chain,
            block,
            u64::from(block_number),
            hash.to_string(),
            timestamp,
        )
        .map(Some)
    }
}

fn hex_be_to_u32(number_hex: &str) -> Result<u32> {
    assert!(number_hex.starts_with("0x"));
    let number_hex = number_hex[2..].to_string();
    assert!(number_hex.len() <= 4 * 2);
    let missing_zeros = (4 * 2) - number_hex.len();
    let number_hex = std::iter::repeat('0')
        .take(missing_zeros)
        .chain(number_hex.chars())
        .collect::<String>();
    let number_bytes = <[u8; 4]>::from_hex(number_hex.as_bytes())?;
    let number = u32::from_be_bytes(number_bytes);
    Ok(number)
}

fn hex_le_to_u64(number_hex: &str) -> Result<u64> {
    assert!(number_hex.starts_with("0x"));
    let number_hex = number_hex[2..].to_string();
    assert!(number_hex.len() <= 8 * 2);
    let number_bytes = <[u8; 8]>::from_hex(number_hex.as_bytes())?;
    let number = u64::from_le_bytes(number_bytes);
    Ok(number)
}

fn storage_key(pallet: &'static str, storage: &'static str) -> Vec<u8> {
    let pallet = sp_core::twox_128(pallet.as_bytes());
    let storage = sp_core::twox_128(storage.as_bytes());

    let mut bytes = vec![];
    bytes.extend(&pallet);
    bytes.extend(&storage);

    bytes
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
    let extrinsics = block
        .get("extrinsics")
        .expect("extrinsics")
        .as_array()
        .expect("array");

    let prev_block_number = block_number.checked_sub(1);
    let num_txs = u64::try_from(extrinsics.len())?;
    let parent_hash = header
        .get("parentHash")
        .expect("parentHash")
        .as_str()
        .expect("str")
        .to_string();

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

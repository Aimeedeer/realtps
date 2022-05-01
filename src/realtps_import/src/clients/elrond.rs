use crate::client::Client;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use realtps_common::{chain::Chain, db::Block};

pub struct ElrondClient {
    client: reqwest::Client,
    url: String,
}

impl ElrondClient {
    pub fn new(url: &str) -> Result<Self> {
        Ok(ElrondClient {
            client: reqwest::Client::new(),
            url: url.to_string(),
        })
    }
}

#[derive(serde::Deserialize, Debug)]
struct ElrondResponse {
    data: serde_json::Value,
    error: Option<String>,
    #[allow(unused)]
    code: serde_json::Value,
}

#[async_trait]
impl Client for ElrondClient {
    async fn client_version(&self) -> Result<String> {
        let url = format!("{}/network/config", self.url);
        let resp = self.client.get(url).send().await?;
        let resp: ElrondResponse = resp.json().await?;
        match (resp.data, resp.error) {
            (serde_json::Value::Null, Some(err)) => Err(anyhow!("{}", err)),
            (serde_json::Value::Null, None) => Err(anyhow!("missing error response")),
            (data, _) => Ok(data
                .get("config")
                .ok_or_else(|| anyhow!("no config key"))?
                .get("erd_latest_tag_software_version")
                .ok_or_else(|| anyhow!("no erd_latest_tag_software_version key"))?
                .as_str()
                .ok_or_else(|| anyhow!("not a string"))?
                .to_string()),
        }
    }

    async fn get_latest_block_number(&self) -> Result<u64> {
        let metablock_shard = 4294967295_u32;
        let url = format!("{}/network/status/{}", self.url, metablock_shard);
        let resp = self.client.get(url).send().await?;
        let resp: ElrondResponse = resp.json().await?;
        match (resp.data, resp.error) {
            (serde_json::Value::Null, Some(err)) => Err(anyhow!("{}", err)),
            (serde_json::Value::Null, None) => Err(anyhow!("missing error response")),
            (data, _) => Ok(data
                .get("status")
                .ok_or_else(|| anyhow!("no status key"))?
                .get("erd_highest_final_nonce")
                .ok_or_else(|| anyhow!("no erd_highest_final_nonce key"))?
                .as_u64()
                .ok_or_else(|| anyhow!("not a u64"))?),
        }
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let url = format!("{}/hyperblock/by-nonce/{}", self.url, block_number);
        let resp = self.client.get(url).send().await?;
        let resp: ElrondResponse = resp.json().await?;
        match (resp.data, resp.error) {
            (serde_json::Value::Null, Some(err)) => Err(anyhow!("{}", err)),
            (serde_json::Value::Null, None) => Err(anyhow!("missing error response")),
            (data, _) => {
                let block = data
                    .get("hyperblock")
                    .ok_or_else(|| anyhow!("no hyperblock key"))?;
                let block_number = block
                    .get("nonce")
                    .ok_or_else(|| anyhow!("no hyperblock key"))?
                    .as_u64()
                    .ok_or_else(|| anyhow!("not a u64"))?;
                let timestamp = block
                    .get("nonce")
                    .ok_or_else(|| anyhow!("no timestamp key"))?
                    .as_u64()
                    .ok_or_else(|| anyhow!("not a u64"))?;
                let num_txs = block
                    .get("numTxs")
                    .ok_or_else(|| anyhow!("no numTxs key"))?
                    .as_u64()
                    .ok_or_else(|| anyhow!("not a u64"))?;
                let hash = block
                    .get("hash")
                    .ok_or_else(|| anyhow!("no hash key"))?
                    .as_str()
                    .ok_or_else(|| anyhow!("not a string"))?
                    .to_string();
                let parent_hash = block
                    .get("prevBlockHash")
                    .ok_or_else(|| anyhow!("no prevBlockHash key"))?
                    .as_str()
                    .ok_or_else(|| anyhow!("not a string"))?
                    .to_string();

                let prev_block_number = Some({
                    let url = format!("{}/hyperblock/by-hash/{}", self.url, parent_hash);
                    let resp = self.client.get(url).send().await?;
                    let resp: ElrondResponse = resp.json().await?;
                    match (resp.data, resp.error) {
                        (serde_json::Value::Null, Some(err)) => Err(anyhow!("{}", err)),
                        (serde_json::Value::Null, None) => Err(anyhow!("missing error response")),
                        (data, _) => {
                            let block = data
                                .get("hyperblock")
                                .ok_or_else(|| anyhow!("no hyperblock key"))?;
                            Ok(block
                                .get("nonce")
                                .ok_or_else(|| anyhow!("no hyperblock key"))?
                                .as_u64()
                                .ok_or_else(|| anyhow!("not a u64"))?)
                        }
                    }?
                });

                Ok(Some(Block {
                    chain: Chain::Elrond,
                    block_number,
                    prev_block_number,
                    timestamp,
                    num_txs,
                    hash,
                    parent_hash,
                }))
            }
        }
    }
}

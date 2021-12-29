#![allow(unused)]

use anyhow::Result;
use crate::client::Client;
use realtps_common::{chain::Chain, db::Block};
use async_trait::async_trait;
use tokio::task;

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

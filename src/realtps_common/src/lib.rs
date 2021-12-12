use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, File};
use std::io::BufWriter;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Chain {
    Ethereum,
    Polygon,
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Chain::Ethereum => write!(f, "ethereum"),
            Chain::Polygon => write!(f, "polygon"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub chain: Chain,
    pub block_number: u64,
    pub timestamp: u64, // seconds since unix epoch
    pub num_txs: u64,
    pub hash: String,
    pub parent_hash: String,
}

pub trait Db {
    fn store_block(&self, block: Block) -> Result<()>;
    fn load_block(&self, block_number: u64) -> Result<Option<Block>>;
    fn get_latest_block_number(&self, chain: Chain) -> Result<u64>;
}

pub struct JsonDb;

pub static JSON_DB_DIR: &'static str = "db";

impl Db for JsonDb {
    fn store_block(&self, block: Block) -> Result<()> {
        let path = format!("{}/{}", JSON_DB_DIR, block.chain);
        fs::create_dir_all(&path)?;

        let path = format!("{}/{}", path, block.block_number);
        let file = File::create(path)?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &block)?;

        Ok(())
    }

    fn load_block(&self, block_number: u64) -> Result<Option<Block>> {
        todo!()
    }

    fn get_latest_block_number(&self, chain: Chain) -> Result<u64> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

#![allow(unused)]

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq)]
#[serde(try_from = "String")]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Ethereum,
    Polygon,
    Avalanche,
    Celo,
    Fantom,
    Moonriver,
    Arbitrum,
    Binance,
    Harmony,
    Rootstock,
}

impl TryFrom<String> for Chain {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "ethereum" => Ok(Chain::Ethereum),
            "polygon" => Ok(Chain::Polygon),
            "avalanche" => Ok(Chain::Avalanche),
            "celo" => Ok(Chain::Celo),
            "fantom" => Ok(Chain::Fantom),
            "moonriver" => Ok(Chain::Moonriver),
            "arbitrum" => Ok(Chain::Arbitrum),
            "binance" => Ok(Chain::Binance),
            "harmony" => Ok(Chain::Harmony),
            "rootstock" => Ok(Chain::Rootstock),
            chain => bail!("failed parsing chain name {}", chain),
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Chain::Ethereum => write!(f, "ethereum"),
            Chain::Polygon => write!(f, "polygon"),
            Chain::Avalanche => write!(f, "avalanche"),
            Chain::Celo => write!(f, "celo"),
            Chain::Fantom => write!(f, "fantom"),
            Chain::Moonriver => write!(f, "moonriver"),
            Chain::Arbitrum => write!(f, "arbitrum"),
            Chain::Binance => write!(f, "binance"),
            Chain::Harmony => write!(f, "harmony"),
            Chain::Rootstock => write!(f, "rootstock"),
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

pub trait Db: Send + Sync + 'static {
    fn store_block(&self, block: Block) -> Result<()>;
    fn load_block(&self, chain: Chain, block_number: u64) -> Result<Option<Block>>;

    fn store_highest_block_number(&self, chain: Chain, block_number: u64) -> Result<()>;
    fn load_highest_block_number(&self, chain: Chain) -> Result<Option<u64>>;

    fn store_tps(&self, chain: Chain, tps: f64) -> Result<()>;
    fn load_tps(&self, chain: Chain) -> Result<Option<f64>>;
}

pub struct JsonDb;

pub static JSON_DB_DIR: &str = "db";
pub static HIGHEST_BLOCK_NUMBER: &str = "heighest_block_number";
pub static TRANSACTIONS_PER_SECOND: &str = "tps";

impl Db for JsonDb {
    fn store_block(&self, block: Block) -> Result<()> {
        let path = format!("{}/{}", JSON_DB_DIR, block.chain);
        fs::create_dir_all(path)?;

        let path = format!("{}/{}/{}", JSON_DB_DIR, block.chain, block.block_number);
        let file = File::create(path)?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &block)?;

        Ok(())
    }

    fn load_block(&self, chain: Chain, block_number: u64) -> Result<Option<Block>> {
        let path = format!("{}/{}/{}", JSON_DB_DIR, chain, block_number);

        let file = File::open(path);
        match file {
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => bail!(e),
            },
            Ok(file) => {
                let reader = BufReader::new(file);
                let block = serde_json::from_reader(reader)?;
                Ok(Some(block))
            }
        }
    }

    fn store_highest_block_number(&self, chain: Chain, block_number: u64) -> Result<()> {
        let path = format!("{}/{}", JSON_DB_DIR, chain);
        fs::create_dir_all(path)?;

        let path = format!("{}/{}/{}", JSON_DB_DIR, chain, HIGHEST_BLOCK_NUMBER);
        let file = File::create(path)?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &block_number)?;

        Ok(())
    }

    fn load_highest_block_number(&self, chain: Chain) -> Result<Option<u64>> {
        let path = format!("{}/{}/{}", JSON_DB_DIR, chain, HIGHEST_BLOCK_NUMBER);

        let file = File::open(path);
        match file {
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => bail!(e),
            },
            Ok(file) => {
                let reader = BufReader::new(file);
                let block_number = serde_json::from_reader(reader)?;
                Ok(Some(block_number))
            }
        }
    }

    fn store_tps(&self, chain: Chain, tps: f64) -> Result<()> {
        let path = format!("{}/{}", JSON_DB_DIR, chain);
        fs::create_dir_all(path)?;

        let path = format!("{}/{}/{}", JSON_DB_DIR, chain, TRANSACTIONS_PER_SECOND);
        let file = File::create(path)?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &tps)?;

        Ok(())
    }

    fn load_tps(&self, chain: Chain) -> Result<Option<f64>> {
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

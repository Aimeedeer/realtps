#![allow(unused)]

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use rand::prelude::*;
use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq)]
#[serde(try_from = "String")]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Arbitrum,
    Avalanche,
    Binance,
    Celo,
    Ethereum,
    Fantom,
    Harmony,
    Moonriver,
    Polygon,
    Rootstock,
}

impl TryFrom<String> for Chain {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "arbitrum" => Ok(Chain::Arbitrum),
            "avalanche" => Ok(Chain::Avalanche),
            "binance" => Ok(Chain::Binance),
            "celo" => Ok(Chain::Celo),
            "ethereum" => Ok(Chain::Ethereum),
            "fantom" => Ok(Chain::Fantom),
            "harmony" => Ok(Chain::Harmony),
            "moonriver" => Ok(Chain::Moonriver),
            "polygon" => Ok(Chain::Polygon),
            "rootstock" => Ok(Chain::Rootstock),
            chain => bail!("failed parsing chain name {}", chain),
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Chain::Arbitrum => write!(f, "arbitrum"),
            Chain::Avalanche => write!(f, "avalanche"),
            Chain::Binance => write!(f, "binance"),
            Chain::Celo => write!(f, "celo"),
            Chain::Ethereum => write!(f, "ethereum"),
            Chain::Fantom => write!(f, "fantom"),
            Chain::Harmony => write!(f, "harmony"),
            Chain::Moonriver => write!(f, "moonriver"),
            Chain::Polygon => write!(f, "polygon"),
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
pub static HIGHEST_BLOCK_NUMBER: &str = "highest_block_number";
pub static TRANSACTIONS_PER_SECOND: &str = "tps";

impl Db for JsonDb {
    fn store_block(&self, block: Block) -> Result<()> {
        write_json_db(
            &format!("{}", block.chain),
            &format!("{}", block.block_number),
            &block,
        )
    }

    fn load_block(&self, chain: Chain, block_number: u64) -> Result<Option<Block>> {
        read_json_db(
            &format!("{}", chain),
            &format!("{}", block_number),
        )
    }

    fn store_highest_block_number(&self, chain: Chain, block_number: u64) -> Result<()> {
        write_json_db(
            &format!("{}", chain),
            &format!("{}", HIGHEST_BLOCK_NUMBER),
            &block_number,
        )
    }

    fn load_highest_block_number(&self, chain: Chain) -> Result<Option<u64>> {
        read_json_db(
            &format!("{}", chain),
            &format!("{}", HIGHEST_BLOCK_NUMBER),
        )
    }

    fn store_tps(&self, chain: Chain, tps: f64) -> Result<()> {
        write_json_db(
            &format!("{}", chain),
            &format!("{}", TRANSACTIONS_PER_SECOND),
            &tps,
        )
    }

    fn load_tps(&self, chain: Chain) -> Result<Option<f64>> {
        read_json_db(
            &format!("{}", chain),
            &format!("{}", TRANSACTIONS_PER_SECOND),
        )
    }
}

fn write_json_db<T>(dir: &str, path: &str, data: &T) -> Result<()> where T: Serialize {
    let file_dir = format!("{}/{}", JSON_DB_DIR, &dir);
    fs::create_dir_all(&file_dir)?;

    let file_path = format!("{}/{}/{}", JSON_DB_DIR, &dir, &path);
    let temp_file_path = format!("{}.{}.temp", &file_path, rand::random::<u32>());

    let file = File::create(&temp_file_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &data)?;

    fs::rename(temp_file_path, file_path)?;
    
    Ok(())
}

fn read_json_db<T>(dir: &str, path: &str) -> Result<Option<T>> where T: DeserializeOwned {
    let path = format!("{}/{}/{}", JSON_DB_DIR, &dir, &path);

    let file = File::open(path);
    match file {
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Ok(None),
            _ => bail!(e),
        },
        Ok(file) => {
            let reader = BufReader::new(file);
            let data = serde_json::from_reader(reader)?;
            Ok(Some(data))
        }
    }
}

pub fn all_chains() -> Vec<Chain> {
    vec![
        Chain::Arbitrum,
        Chain::Avalanche,
        Chain::Binance,
        Chain::Celo,
        Chain::Ethereum,
        Chain::Fantom,
        Chain::Harmony,
        Chain::Moonriver,
        Chain::Polygon,
        Chain::Rootstock,
    ]
}

pub fn chain_description(chain: Chain) -> &'static str {
    match chain {
        Chain::Arbitrum => "Arbitrum",
        Chain::Avalanche => "Avalanche C-Chain",
        Chain::Binance => "Binance Smart Chain",
        Chain::Celo => "Celo",
        Chain::Ethereum => "Ethereum",
        Chain::Fantom => "Fantom",
        Chain::Harmony => "Harmony",
        Chain::Moonriver => "Moonriver",
        Chain::Polygon => "Polygon PoS",
        Chain::Rootstock => "Rootstock",
    }
}

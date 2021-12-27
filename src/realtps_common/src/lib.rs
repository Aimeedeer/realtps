use anyhow::{bail, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub chain: Chain,
    pub block_number: u64,
    /// The previous block number, not always block_number - 1, as in Solana,
    /// where the "block" number is really a "slot" number, and slots may be
    /// empty.
    pub prev_block_number: Option<u64>,
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
        read_json_db(&format!("{}", chain), &format!("{}", block_number))
    }

    fn store_highest_block_number(&self, chain: Chain, block_number: u64) -> Result<()> {
        write_json_db(
            &format!("{}", chain),
            &format!("{}", HIGHEST_BLOCK_NUMBER),
            &block_number,
        )
    }

    fn load_highest_block_number(&self, chain: Chain) -> Result<Option<u64>> {
        read_json_db(&format!("{}", chain), &format!("{}", HIGHEST_BLOCK_NUMBER))
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

fn write_json_db<T>(dir: &str, path: &str, data: &T) -> Result<()>
where
    T: Serialize,
{
    let file_dir = format!("{}/{}", JSON_DB_DIR, &dir);
    fs::create_dir_all(&file_dir)?;

    let file_path = format!("{}/{}/{}", JSON_DB_DIR, &dir, &path);
    let temp_file_path = format!("{}.{}.temp", &file_path, rand::random::<u32>());

    let file = File::create(&temp_file_path)?;
    let mut writer = BufWriter::new(file);

    match serde_json::to_writer(&mut writer, &data) {
        Err(e) => {
            fs::remove_file(temp_file_path)?;
            bail!(e)
        }
        Ok(()) => {
            fs::rename(temp_file_path, file_path)?;
            Ok(())
        }
    }
}

fn read_json_db<T>(dir: &str, path: &str) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq)]
#[serde(try_from = "String")]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Arbitrum,
    Avalanche,
    Binance,
    Celo,
    CosmosHub,
    Cronos,
    Ethereum,
    Fantom,
    Harmony,
    Heco,
    KuCoin,
    Moonriver,
    Near,
    OKEx,
    Optimism,
    Polygon,
    Rootstock,
    SecretNetwork,
    Solana,
    Terra,
    XDai,
}

// for serde deserializing
impl<'a> TryFrom<&'a str> for Chain {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self> {
        match value {
            "arbitrum" => Ok(Chain::Arbitrum),
            "avalanche" => Ok(Chain::Avalanche),
            "binance" => Ok(Chain::Binance),
            "celo" => Ok(Chain::Celo),
            "cosmoshub" => Ok(Chain::CosmosHub),
            "cronos" => Ok(Chain::Cronos),
            "ethereum" => Ok(Chain::Ethereum),
            "fantom" => Ok(Chain::Fantom),
            "harmony" => Ok(Chain::Harmony),
            "heco" => Ok(Chain::Heco),
            "kucoin" => Ok(Chain::KuCoin),
            "moonriver" => Ok(Chain::Moonriver),
            "near" => Ok(Chain::Near),
            "okex" => Ok(Chain::OKEx),
            "optimism" => Ok(Chain::Optimism),
            "polygon" => Ok(Chain::Polygon),
            "rootstock" => Ok(Chain::Rootstock),
            "secretnetwork" => Ok(Chain::SecretNetwork),
            "solana" => Ok(Chain::Solana),
            "terra" => Ok(Chain::Terra),
            "xdai" => Ok(Chain::XDai),
            chain => bail!("failed parsing chain name {}", chain),
        }
    }
}

// for parsing command line used in `structopt`
impl TryFrom<String> for Chain {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        Chain::try_from(value.as_ref())
    }
}

// for serde serializing
impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Chain::Arbitrum => write!(f, "arbitrum"),
            Chain::Avalanche => write!(f, "avalanche"),
            Chain::Binance => write!(f, "binance"),
            Chain::Celo => write!(f, "celo"),
            Chain::CosmosHub => write!(f, "cosmoshub"),
            Chain::Cronos => write!(f, "cronos"),
            Chain::Ethereum => write!(f, "ethereum"),
            Chain::Fantom => write!(f, "fantom"),
            Chain::Harmony => write!(f, "harmony"),
            Chain::Heco => write!(f, "heco"),
            Chain::KuCoin => write!(f, "kucoin"),
            Chain::Moonriver => write!(f, "moonriver"),
            Chain::Near => write!(f, "near"),
            Chain::OKEx => write!(f, "okex"),
            Chain::Optimism => write!(f, "optimism"),
            Chain::Polygon => write!(f, "polygon"),
            Chain::Rootstock => write!(f, "rootstock"),
            Chain::SecretNetwork => write!(f, "secretnetwork"),
            Chain::Solana => write!(f, "solana"),
            Chain::Terra => write!(f, "terra"),
            Chain::XDai => write!(f, "xdai"),
        }
    }
}

// Chain names showed on the website
pub fn chain_description(chain: Chain) -> &'static str {
    match chain {
        Chain::Arbitrum => "Arbitrum",
        Chain::Avalanche => "Avalanche C-Chain",
        Chain::Binance => "Binance Smart Chain",
        Chain::Celo => "Celo",
        Chain::CosmosHub => "Cosmos Hub",
        Chain::Cronos => "Cronos",
        Chain::Ethereum => "Ethereum",
        Chain::Fantom => "Fantom",
        Chain::Harmony => "Harmony",
        Chain::Heco => "Heco",
        Chain::KuCoin => "KuCoin",
        Chain::Moonriver => "Moonriver",
        Chain::Near => "NEAR",
        Chain::OKEx => "OKEx",
        Chain::Optimism => "Optimism",
        Chain::Polygon => "Polygon PoS",
        Chain::Rootstock => "Rootstock",
        Chain::SecretNetwork => "Secret Network",
        Chain::Solana => "Solana",
        Chain::Terra => "Terra",
        Chain::XDai => "xDai",
    }
}

pub fn all_chains() -> Vec<Chain> {
    vec![
        Chain::Arbitrum,
        Chain::Avalanche,
        Chain::Binance,
        Chain::Celo,
        Chain::CosmosHub,
        Chain::Cronos,
        Chain::Ethereum,
        Chain::Fantom,
        Chain::Harmony,
        Chain::Heco,
        Chain::KuCoin,
        Chain::Moonriver,
        Chain::Near,
        Chain::OKEx,
        Chain::Optimism,
        Chain::Polygon,
        Chain::Rootstock,
        Chain::SecretNetwork,
        Chain::Solana,
        Chain::Terra,
        Chain::XDai,
    ]
}

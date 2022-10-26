use crate::chain::Chain;
use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
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
    // FIXME this could be None, like prev_block_number
    pub parent_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CalculationLog {
    pub calculating_start: DateTime<Utc>,
    pub calculating_end: DateTime<Utc>,
    pub newest_block_timestamp: DateTime<Utc>,
    pub oldest_block_timestamp: DateTime<Utc>,
}

pub trait Db: Send + Sync + 'static {
    fn store_block(&self, block: Block) -> Result<()>;
    fn load_block(&self, chain: Chain, block_number: u64) -> Result<Option<Block>>;

    fn store_highest_block_number(&self, chain: Chain, block_number: u64) -> Result<()>;
    fn load_highest_block_number(&self, chain: Chain) -> Result<Option<u64>>;

    fn store_tps(&self, chain: Chain, tps: f64) -> Result<()>;
    fn load_tps(&self, chain: Chain) -> Result<Option<f64>>;

    fn remove_block(&self, chain: Chain, block: u64) -> Result<()>;

    fn store_calculation_log(&self, chain: Chain, log: &CalculationLog) -> Result<()>;

    fn load_calculation_log(&self, chain: Chain) -> Result<Option<CalculationLog>>;
}

pub struct JsonDb;

pub static JSON_DB_DIR: &str = "db";
pub static DB_DIR_BLOCKS: &str = "blocks";
pub static DB_DIR_META: &str = "meta";
pub static HIGHEST_BLOCK_NUMBER: &str = "highest_block_number";
pub static TRANSACTIONS_PER_SECOND: &str = "tps";
pub static CALCULATION_LOG: &str = "calculation_log";

impl Db for JsonDb {
    fn store_block(&self, block: Block) -> Result<()> {
        write_json_db(
            &format!("{}", block.chain),
            DB_DIR_BLOCKS,
            &format!("{}", block.block_number),
            &block,
        )
    }

    fn load_block(&self, chain: Chain, block_number: u64) -> Result<Option<Block>> {
        read_json_db(
            &format!("{}", chain),
            DB_DIR_BLOCKS,
            &format!("{}", block_number),
        )
    }

    fn store_highest_block_number(&self, chain: Chain, block_number: u64) -> Result<()> {
        write_json_db(
            &format!("{}", chain),
            DB_DIR_META,
            HIGHEST_BLOCK_NUMBER,
            &block_number,
        )
    }

    fn load_highest_block_number(&self, chain: Chain) -> Result<Option<u64>> {
        read_json_db(&format!("{}", chain), DB_DIR_META, HIGHEST_BLOCK_NUMBER)
    }

    fn store_tps(&self, chain: Chain, tps: f64) -> Result<()> {
        write_json_db(
            &format!("{}", chain),
            DB_DIR_META,
            TRANSACTIONS_PER_SECOND,
            &tps,
        )
    }

    fn load_tps(&self, chain: Chain) -> Result<Option<f64>> {
        read_json_db(&format!("{}", chain), DB_DIR_META, TRANSACTIONS_PER_SECOND)
    }

    fn remove_block(&self, chain: Chain, block: u64) -> Result<()> {
        let file_path = format!("{}/{}/{}/{}", JSON_DB_DIR, chain, DB_DIR_BLOCKS, block);
        fs::remove_file(file_path)?;
        Ok(())
    }

    fn store_calculation_log(&self, chain: Chain, log: &CalculationLog) -> Result<()> {
        write_json_db(&format!("{}", chain), DB_DIR_META, CALCULATION_LOG, log)
    }

    fn load_calculation_log(&self, chain: Chain) -> Result<Option<CalculationLog>> {
        read_json_db(&format!("{}", chain), DB_DIR_META, CALCULATION_LOG)
    }
}

fn write_json_db<T>(chain: &str, sub_dir: &str, file: &str, data: &T) -> Result<()>
where
    T: Serialize + ?Sized,
{
    let file_dir = format!("{}/{}/{}", JSON_DB_DIR, &chain, &sub_dir);
    fs::create_dir_all(&file_dir)?;

    let file_path = format!("{}/{}/{}/{}", JSON_DB_DIR, &chain, &sub_dir, &file);
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

fn read_json_db<T>(chain: &str, sub_dir: &str, file: &str) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let path = format!("{}/{}/{}/{}", JSON_DB_DIR, &chain, &sub_dir, &file);

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

use anyhow::{bail, Result};
use clap;
use serde::{Deserialize, Serialize};
use std::fmt;

pub enum ChainType {
    Ethers,
    Near,
    Solana,
    Tendermint,
    Substrate,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq)]
#[serde(try_from = "String")]
#[serde(rename_all = "lowercase")]
#[derive(clap::ArgEnum)]
pub enum Chain {
    Arbitrum,
    Avalanche,
    Binance,
    Celo,
//    CosmosHub,
    Cronos,
    Ethereum,
    Fantom,
    Harmony,
    Heco,
    KuCoin,
    Kusama,
    Moonriver,
    Near,
    OKEx,
    Optimism,
    Osmosis,
    Polkadot,
    Polygon,
    Rootstock,
    SecretNetwork,
    Solana,
    Terra,
    XDai,
}

impl Chain {
    pub fn all_chains() -> Vec<Chain> {
        vec![
            Chain::Arbitrum,
            Chain::Avalanche,
            Chain::Binance,
            Chain::Celo,
//            Chain::CosmosHub,
            Chain::Cronos,
            Chain::Ethereum,
            Chain::Fantom,
            Chain::Harmony,
            Chain::Heco,
            Chain::KuCoin,
            Chain::Kusama,
            Chain::Moonriver,
            Chain::Near,
            Chain::OKEx,
            Chain::Optimism,
            Chain::Osmosis,
            Chain::Polkadot,
            Chain::Polygon,
            Chain::Rootstock,
            Chain::SecretNetwork,
            Chain::Solana,
            Chain::Terra,
            Chain::XDai,
        ]
    }

    /// Chain names showed on the website
    pub fn description(&self) -> &'static str {
        match *self {
            Chain::Arbitrum => "Arbitrum",
            Chain::Avalanche => "Avalanche C-Chain",
            Chain::Binance => "Binance Smart Chain",
            Chain::Celo => "Celo",
//            Chain::CosmosHub => "Cosmos Hub",
            Chain::Cronos => "Cronos",
            Chain::Ethereum => "Ethereum",
            Chain::Fantom => "Fantom",
            Chain::Harmony => "Harmony",
            Chain::Heco => "Heco",
            Chain::KuCoin => "KuCoin",
            Chain::Kusama => "Kusama",
            Chain::Moonriver => "Moonriver",
            Chain::Near => "NEAR",
            Chain::OKEx => "OKEx",
            Chain::Optimism => "Optimism",
            Chain::Osmosis => "Osmosis",
            Chain::Polkadot => "Polkadot",
            Chain::Polygon => "Polygon PoS",
            Chain::Rootstock => "Rootstock",
            Chain::SecretNetwork => "Secret Network",
            Chain::Solana => "Solana",
            Chain::Terra => "Terra",
            Chain::XDai => "xDai",
        }
    }

    pub fn chain_type(&self) -> ChainType {
        match self {
            Chain::Arbitrum
            | Chain::Avalanche
            | Chain::Binance
            | Chain::Celo
            | Chain::Cronos
            | Chain::Ethereum
            | Chain::Fantom
            | Chain::Harmony
            | Chain::Heco
            | Chain::KuCoin
            | Chain::Moonriver
            | Chain::OKEx
            | Chain::Optimism
            | Chain::Polygon
            | Chain::Rootstock
            | Chain::XDai => ChainType::Ethers,
            Chain::Near => ChainType::Near,
            Chain::Solana => ChainType::Solana,
            // Chain::CosmosHub |
            Chain::Osmosis | Chain::SecretNetwork | Chain::Terra => ChainType::Tendermint,
            Chain::Kusama | Chain::Polkadot => ChainType::Substrate,
        }
    }
}

// For parsing command line used in `structopt`.
impl<'a> TryFrom<&'a str> for Chain {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self> {
        match value {
            "arbitrum" => Ok(Chain::Arbitrum),
            "avalanche" => Ok(Chain::Avalanche),
            "binance" => Ok(Chain::Binance),
            "celo" => Ok(Chain::Celo),
//            "cosmoshub" => Ok(Chain::CosmosHub),
            "cronos" => Ok(Chain::Cronos),
            "ethereum" => Ok(Chain::Ethereum),
            "fantom" => Ok(Chain::Fantom),
            "harmony" => Ok(Chain::Harmony),
            "heco" => Ok(Chain::Heco),
            "kucoin" => Ok(Chain::KuCoin),
            "kusama" => Ok(Chain::Kusama),
            "moonriver" => Ok(Chain::Moonriver),
            "near" => Ok(Chain::Near),
            "okex" => Ok(Chain::OKEx),
            "optimism" => Ok(Chain::Optimism),
            "osmosis" => Ok(Chain::Osmosis),
            "polkadot" => Ok(Chain::Polkadot),
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

// For serde deserializing.
impl TryFrom<String> for Chain {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        Chain::try_from(value.as_ref())
    }
}

// Displays a "chain id". Used in `JsonDb` paths and logging.
impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

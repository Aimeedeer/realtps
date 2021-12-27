use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

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

// For parsing command line used in `structopt`.
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

pub enum ChainType {
    Ethers,
    Near,
    Solana,
    Tendermint,
}

impl Chain {
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
            Chain::CosmosHub | Chain::SecretNetwork | Chain::Terra => ChainType::Tendermint,
        }
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

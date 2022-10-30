use anyhow::{bail, Result};
use clap;
use serde::{Deserialize, Serialize};
use std::fmt;

pub enum ChainType {
    Algorand,
    Esplora, // Bitcoin
    Elrond,
    Ethers,
    Hedera,
    InternetComputer,
    Near,
    Solana,
    Stellar,
    Substrate,
    Tendermint,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq)]
#[serde(try_from = "String")]
#[serde(rename_all = "lowercase")]
#[derive(clap::ArgEnum)]
pub enum Chain {
    Acala,
    Algorand,
    Arbitrum,
    Astar,
    Avalanche,
    Bifrost,
    Binance,
    Bitcoin,
    Celo,
    CosmosHub,
    Cronos,
    Elrond,
    Ethereum,
    Fantom,
    Harmony,
    Hedera,
    Heco,
    InternetComputer,
    Karura,
    KuCoin,
    Kusama,
    Moonbeam,
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
    Stellar,
    Terra,
}

impl Chain {
    pub fn all_chains() -> Vec<Chain> {
        vec![
            Chain::Acala,
            Chain::Algorand,
            Chain::Arbitrum,
            Chain::Astar,
            Chain::Avalanche,
            Chain::Bifrost,
            Chain::Binance,
            Chain::Bitcoin,
            Chain::Celo,
            // todo rpc disappeared
            // Chain::CosmosHub,
            Chain::Cronos,
            Chain::Elrond,
            Chain::Ethereum,
            Chain::Fantom,
            Chain::Harmony,
            Chain::InternetComputer,
            Chain::Hedera,
            // todo ssl handshake failure
            // Chain::Heco,
            Chain::Karura,
            Chain::KuCoin,
            Chain::Kusama,
            Chain::Moonbeam,
            Chain::Moonriver,
            Chain::Near,
            Chain::OKEx,
            Chain::Optimism,
            // todo banned
            // Chain::Osmosis,
            Chain::Polkadot,
            Chain::Polygon,
            Chain::Rootstock,
            // todo banned?
            // Chain::SecretNetwork,
            Chain::Solana,
            Chain::Stellar,
            // todo forked, rpc disappeared
            //Chain::Terra,
        ]
    }

    /// Chain names showed on the website
    pub fn description(&self) -> &'static str {
        match *self {
            Chain::Acala => "Acala",
            Chain::Algorand => "Algorand",
            Chain::Arbitrum => "Arbitrum",
            Chain::Astar => "Astar",
            Chain::Avalanche => "Avalanche C-Chain",
            Chain::Bifrost => "Bifrost",
            Chain::Binance => "Binance Smart Chain",
            Chain::Bitcoin => "Bitcoin",
            Chain::Celo => "Celo",
            Chain::CosmosHub => "Cosmos Hub",
            Chain::Cronos => "Cronos",
            Chain::Elrond => "Elrond",
            Chain::Ethereum => "Ethereum",
            Chain::Fantom => "Fantom",
            Chain::Harmony => "Harmony",
            Chain::Hedera => "Hedera",
            Chain::Heco => "Heco",
            Chain::InternetComputer => "Internet Computer",
            Chain::Karura => "Karura",
            Chain::KuCoin => "KuCoin",
            Chain::Kusama => "Kusama",
            Chain::Moonbeam => "Moonbeam",
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
            Chain::Stellar => "Stellar",
            Chain::Terra => "Terra",
        }
    }

    pub fn chain_type(&self) -> ChainType {
        match self {
            Chain::Arbitrum
            | Chain::Astar
            | Chain::Avalanche
            | Chain::Binance
            | Chain::Celo
            | Chain::Cronos
            | Chain::Ethereum
            | Chain::Fantom
            | Chain::Harmony
            | Chain::Heco
            | Chain::KuCoin
            | Chain::Moonbeam
            | Chain::Moonriver
            | Chain::OKEx
            | Chain::Optimism
            | Chain::Polygon
            | Chain::Rootstock => ChainType::Ethers,
            Chain::Bitcoin => ChainType::Esplora,
            Chain::Elrond => ChainType::Elrond,
            Chain::Hedera => ChainType::Hedera,
            Chain::InternetComputer => ChainType::InternetComputer,
            Chain::Near => ChainType::Near,
            Chain::Solana => ChainType::Solana,
            Chain::Stellar => ChainType::Stellar,
            Chain::CosmosHub | Chain::Osmosis | Chain::SecretNetwork | Chain::Terra => {
                ChainType::Tendermint
            }
            Chain::Acala | Chain::Bifrost | Chain::Karura | Chain::Kusama | Chain::Polkadot => {
                ChainType::Substrate
            }
            Chain::Algorand => ChainType::Algorand,
        }
    }
}

// For serde deserializing.
// Needs to be the same as whatever serde serializes.
impl<'a> TryFrom<&'a str> for Chain {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self> {
        match value {
            "acala" => Ok(Chain::Acala),
            "algorand" => Ok(Chain::Algorand),
            "arbitrum" => Ok(Chain::Arbitrum),
            "astar" => Ok(Chain::Astar),
            "avalanche" => Ok(Chain::Avalanche),
            "bifrost" => Ok(Chain::Bifrost),
            "binance" => Ok(Chain::Binance),
            "bitcoin" => Ok(Chain::Bitcoin),
            "celo" => Ok(Chain::Celo),
            "cosmoshub" => Ok(Chain::CosmosHub),
            "cronos" => Ok(Chain::Cronos),
            "elrond" => Ok(Chain::Elrond),
            "ethereum" => Ok(Chain::Ethereum),
            "fantom" => Ok(Chain::Fantom),
            "harmony" => Ok(Chain::Harmony),
            "hedera" => Ok(Chain::Hedera),
            "heco" => Ok(Chain::Heco),
            "internetcomputer" => Ok(Chain::InternetComputer),
            "karura" => Ok(Chain::Karura),
            "kucoin" => Ok(Chain::KuCoin),
            "kusama" => Ok(Chain::Kusama),
            "moonbeam" => Ok(Chain::Moonbeam),
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
            "stellar" => Ok(Chain::Stellar),
            "terra" => Ok(Chain::Terra),
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

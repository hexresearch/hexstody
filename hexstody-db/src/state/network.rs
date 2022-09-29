use bitcoin::network::constants::Network as BtcNetwork;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema,
)]
pub enum Network {
    Mainnet,
    Testnet,
    Regtest,
}

impl Network {
    pub fn btc(&self) -> BtcNetwork {
        match self {
            Network::Mainnet => BtcNetwork::Bitcoin,
            Network::Testnet => BtcNetwork::Testnet,
            Network::Regtest => BtcNetwork::Regtest,
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Testnet => write!(f, "testnet"),
            Network::Regtest => write!(f, "regtest"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct UnknownNetwork(String);

impl std::error::Error for UnknownNetwork {}

impl fmt::Display for UnknownNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Given network '{}' is unknown, valid are: mainnet, testnet, regtest",
            self.0
        )
    }
}

impl FromStr for Network {
    type Err = UnknownNetwork;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "mainnet" => Ok(Network::Mainnet),
            "testnet" => Ok(Network::Testnet),
            "regtest" => Ok(Network::Regtest),
            _ => Err(UnknownNetwork(s.to_owned())),
        }
    }
}

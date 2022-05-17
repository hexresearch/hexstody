use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A currency that custody understands. Can be extended in future.
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Currency {
    BTC,
    ETH,
    ERC20(Erc20Token),
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Currency::BTC => write!(f, "Bitcoin"),
            Currency::ETH => write!(f, "Ethereum"),
            Currency::ERC20(token) => write!(f, "{} ERC-20", token),
        }
    }
}

impl Currency {
    /// List supported currencies at the moment
    pub fn supported() -> Vec<Currency> {
        vec![Currency::BTC, Currency::ETH]
    }
}

/// Description of ERC20 token that allows to distinguish them between each other
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Erc20Token {
    /// Short name of the token like USDT or WBTC
    pub ticker: String,
    /// Long name like 'Wrapped Bitcoin'
    pub name: String,
    /// Contract address
    pub contract: String,
}

impl fmt::Display for Erc20Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ticker)
    }
}

/// Address that can be used for receiving currency
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CurrencyAddress {
    BTC(BtcAddress),
    ETH(EthAccount),
    ERC20(Erc20Token, EthAccount),
}

impl CurrencyAddress {
    pub fn currency(&self) -> Currency {
        match self {
            CurrencyAddress::BTC(_) => Currency::BTC,
            CurrencyAddress::ETH(_) => Currency::ETH,
            CurrencyAddress::ERC20(token, _) => Currency::ERC20(token.clone()),
        }
    }
}

impl fmt::Display for CurrencyAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrencyAddress::BTC(addr) => write!(f, "{}", addr),
            CurrencyAddress::ETH(acc) => write!(f, "{}", acc),
            CurrencyAddress::ERC20(_, acc) => write!(f, "{}", acc),
        }
    }
}

/// Validated bitcoin address
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BtcAddress(pub String);

impl fmt::Display for BtcAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validated ethereum account address
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EthAccount(pub String);

impl fmt::Display for EthAccount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

use bitcoin;
use rocket::request::FromParam;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, vec, str::FromStr, num::ParseIntError};

use crate::types::TokenInfo;

/// A currency that custody understands. Can be extended in future.
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Currency {
    BTC,
    ETH,
    USDT,
    GTECH,
    CRV,
}

fn erc20_usdt() -> Erc20Token {
    Erc20Token {
        ticker: Currency::USDT,
        name: "USDT".to_string(),
        contract: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
    }
}

fn erc20_crv() -> Erc20Token {
    Erc20Token {
        ticker: Currency::CRV,
        name: "CRV".to_string(),
        contract: "0xD533a949740bb3306d119CC777fa900bA034cd52".to_string(),
    }
}

fn erc20_gtech() -> Erc20Token {
    Erc20Token {
        ticker: Currency::GTECH,
        name: "GTECH".to_string(),
        contract: "0xD533a949740bb3306d119CC777fa900bA034cd52".to_string(),
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            c if c.is_token() => write!(f, "{} ERC-20", c),
            Currency::BTC => write!(f, "Bitcoin"),
            Currency::ETH => write!(f, "Ethereum"),
            c => panic!("Cannot display {}", c),
        }
    }
}

impl FromStr for Currency {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BTC" => Ok(Currency::BTC),
            "USDT" => Ok(Currency::USDT),
            "ETH" => Ok(Currency::ETH),
            "GTECH" => Ok(Currency::GTECH),
            "CRV" => Ok(Currency::CRV),
             _  => panic!("unknown currency {}", s)
        }
    }
}

impl<'r> FromParam<'r> for Currency {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        param.parse().map_err(|_| param)
    }
}

impl Currency {
    pub fn f(&self) -> Erc20Token {
        match self {
            Currency::USDT => erc20_usdt(),
            Currency::GTECH => erc20_gtech(),
            Currency::CRV => erc20_crv(),
            _ => panic!("{} is not an ERC20 token", self)
        }
    }
    /// List supported currencies at the moment
    pub fn supported() -> Vec<Currency> {
        vec![
            Currency::BTC,
            Currency::ETH,
            Currency::USDT,
            Currency::CRV,
            Currency::GTECH,
        ]
    }

    /// Check if the currency is a token
    pub fn is_token(&self) -> bool {
        match self {
            Currency::USDT | Currency::GTECH | Currency::CRV => true,
            _ => false,
        }
    }

    pub fn supported_tokens() -> Vec<Erc20Token> {
        vec![
            Currency::USDT.f().to_owned(),
            Currency::GTECH.f().to_owned(),
            Currency::CRV.f().to_owned(),
        ]
    }

    pub fn default_tokens() -> Vec<Erc20Token> {
        vec![
            Currency::USDT.f().to_owned(),
            Currency::GTECH.f().to_owned(),
        ]
    }

    /// List of currencies active by default for a new user
    pub fn default_currencies() -> Vec<Currency> {
        vec![
            Currency::BTC,
            Currency::ETH,
            Currency::USDT,
            Currency::GTECH,
        ]
    }
}

pub fn filter_tokens(curs: Vec<Currency>) -> Vec<Erc20Token> {
    curs.into_iter()
        .filter_map(|c| if c.is_token() { Some(c.f()) } else { None })
        .collect()
}

/// Description of ERC20 token that allows to distinguish them between each other
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Erc20Token {
    /// Short name of the token like USDT or WBTC
    pub ticker: Currency,
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
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(tag = "type")]
pub enum CurrencyAddress {
    BTC(BtcAddress),
    ETH(EthAccount),
    ERC20(Erc20),
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Erc20 {
    token: Erc20Token,
    account: EthAccount,
}

impl CurrencyAddress {
    pub fn currency(&self) -> Currency {
        match self {
            CurrencyAddress::BTC(_) => Currency::BTC,
            CurrencyAddress::ETH(_) => Currency::ETH,
            CurrencyAddress::ERC20(erc20) => erc20.token.ticker.to_owned(),
        }
    }
}

impl fmt::Display for CurrencyAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrencyAddress::BTC(addr) => write!(f, "{}", addr),
            CurrencyAddress::ETH(acc) => write!(f, "{}", acc),
            CurrencyAddress::ERC20(erc20) => write!(f, "{}", erc20.account),
        }
    }
}

impl From<bitcoin::Address> for CurrencyAddress {
    fn from(addr: bitcoin::Address) -> Self {
        CurrencyAddress::BTC(BtcAddress {
            addr: addr.to_string(),
        })
    }
}

/// Validated bitcoin address
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct BtcAddress {
    pub addr: String,
}

impl fmt::Display for BtcAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.addr)
    }
}

/// Validated ethereum account address
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct EthAccount {
    pub account: String,
}

impl fmt::Display for EthAccount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.account)
    }
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct BTCTxid {
    pub txid: String,
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct ETHTxid {
    pub txid: String,
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(tag = "type")]
pub enum CurrencyTxId {
    BTC(BTCTxid),
    ETH(ETHTxid),
}

impl CurrencyTxId {
    pub fn currency(&self) -> Currency {
        match self {
            CurrencyTxId::BTC(_) => Currency::BTC,
            CurrencyTxId::ETH(_) => Currency::ETH,
        }
    }
}

impl fmt::Display for CurrencyTxId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrencyTxId::BTC(BTCTxid { txid }) => write!(f, "{}", txid),
            CurrencyTxId::ETH(ETHTxid { txid }) => write!(f, "{}", txid),
        }
    }
}

impl From<bitcoin::Txid> for CurrencyTxId {
    fn from(txid: bitcoin::Txid) -> Self {
        CurrencyTxId::BTC(BTCTxid {
            txid: txid.to_string(),
        })
    }
}

use bitcoin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::char::ParseCharError;
use std::convert::Infallible;
use std::str::FromStr;
use std::string::ParseError;
use std::{fmt, vec};

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

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum CurrencyCode {
    BTC,
    ETH,
    USDT,
    CRV,
    GTECH,
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrencyCode::BTC => write!(f, "BTC"),
            CurrencyCode::ETH => write!(f, "ETH"),
            CurrencyCode::USDT => write!(f, "USDT"),
            CurrencyCode::GTECH => write!(f, "GTECH"),
            CurrencyCode::CRV => write!(f, "CRV"),
        }
    }
}

use rocket::request::FromParam;

impl<'r> FromParam<'r> for CurrencyCode {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        CurrencyCode::from_str(param).map_err(|_| param)
    }
}


impl FromStr for CurrencyCode {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        

        match s {
            "BTC" => Ok(CurrencyCode::BTC),
            "ETH" => Ok(CurrencyCode::ETH),
            "USDT" => Ok(CurrencyCode::USDT),
            "CRV" => Ok(CurrencyCode::CRV),
            "GTECH" => Ok(CurrencyCode::GTECH),
            _ => todo!("error"),
        }
    }
}

impl Currency {
    /// List supported currencies at the moment
    pub fn supported() -> Vec<Currency> {
        vec![
            Currency::BTC,
            Currency::ETH,
            Currency::ERC20(Erc20Token {
                ticker: CurrencyCode::USDT,
                name: "USDT".to_string(),
                contract: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            }),
            Currency::ERC20(Erc20Token {
                ticker: CurrencyCode::CRV,
                name: "CRV".to_string(),
                contract: "0xD533a949740bb3306d119CC777fa900bA034cd52".to_string(),
            }),
            Currency::ERC20(Erc20Token {
                ticker: CurrencyCode::GTECH,
                name: CurrencyCode::GTECH.to_string(),
                contract: "0xD533a949740bb3306d119CC777fa900bA034cd52".to_string(),
            }),
        ]
    }

    /// Check if the currency is a token
    pub fn is_token(&self) -> bool {
        match self {
            Currency::ERC20(_) => true,
            _ => false,
        }
    }

    pub fn supported_tokens() -> Vec<Erc20Token> {
        Currency::supported()
            .into_iter()
            .filter_map(|c| match c {
                Currency::ERC20(token) => Some(token),
                _ => None,
            })
            .collect()
    }

    pub fn default_tokens() -> Vec<Erc20Token> {
        let supported_tickers = vec![CurrencyCode::USDT, CurrencyCode::GTECH];
        Currency::supported()
            .into_iter()
            .filter_map(|c| match c {
                Currency::ERC20(token) => {
                    if supported_tickers.contains(&token.ticker) {
                        Some(token)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect()
    }

    /// List of currencies active by default for a new user
    pub fn default_currencies() -> Vec<Currency> {
        Currency::supported()
            .into_iter()
            .filter_map(|c| match c.clone() {
                Currency::ERC20(token) => {
                    if token.ticker == CurrencyCode::CRV {
                        None
                    } else {
                        Some(c)
                    }
                }
                _ => Some(c),
            })
            .collect()
    }

    pub fn currency_code(x: Currency) -> CurrencyCode {
        match x {
            Currency::BTC => CurrencyCode::BTC,
            Currency::ETH => CurrencyCode::ETH,
            Currency::ERC20(token) => token.ticker,
        }
    }
}

pub fn filter_tokens(curs: Vec<Currency>) -> Vec<Erc20Token> {
    curs.into_iter()
        .filter_map(|c| match c {
            Currency::ERC20(token) => Some(token),
            _ => None,
        })
        .collect()
}

/// Description of ERC20 token that allows to distinguish them between each other
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Erc20Token {
    /// Short name of the token like USDT or WBTC
    pub ticker: CurrencyCode,
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
            CurrencyAddress::ERC20(erc20) => Currency::ERC20(erc20.token.clone()),
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

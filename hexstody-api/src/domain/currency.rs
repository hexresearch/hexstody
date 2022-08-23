use bitcoin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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

impl Currency {
    pub fn usdt_erc20() -> Currency {
        Currency::ERC20(Erc20Token {
            ticker: "USDT".to_string(),
            name: "USDT".to_string(),
            contract: "0x5bF7700B03631a8D917446586Df091CF72F6ebf0".to_string(),
        })
    }

    pub fn crv_erc20() -> Currency {
        Currency::ERC20(Erc20Token {
            ticker: "CRV".to_string(),
            name: "CRV".to_string(),
            contract: "0x7413679bCD0B2cD7c1492Bf9Ca8743f64316a582".to_string(),
        })
    }

    pub fn gtech_erc20() -> Currency {
        Currency::ERC20(Erc20Token {
            ticker: "GTECH".to_string(),
            name: "GTECH".to_string(),
            contract: "0xD533a949740bb3306d119CC777fa900bA034cd52".to_string(),
        })
    }

    /// Check if the currency is a token
    pub fn ticker_lowercase(&self) -> String {
        match self {
            Currency::BTC => "btc".to_owned(),
            Currency::ETH => "eth".to_owned(),
            Currency::ERC20(token) => token.ticker.to_lowercase(),
        }
    }

    /// List supported currencies at the moment
    pub fn supported() -> Vec<Currency> {
        vec![
            Currency::BTC,
            Currency::ETH,
            Currency::usdt_erc20(),
            Currency::crv_erc20(),
            Currency::gtech_erc20(),
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
        let supported_tickers = vec!["USDT".to_string(), "GTECH".to_string()];
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
                    if token.ticker == "CRV" {
                        None
                    } else {
                        Some(c)
                    }
                }
                _ => Some(c),
            })
            .collect()
    }

    pub fn get_by_name(name_orig: &str) -> Option<Currency>{
        let name = name_orig.to_uppercase();
        if name == "BTC" {
            return Some(Currency::BTC)
        } else if name == "ETH" {
            return Some(Currency::ETH);
        } else {
            let tokens = Currency::supported_tokens();
            for token in tokens {
                if name == token.ticker{
                    return Some(Currency::ERC20(token))
                };
            }
            return None;
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

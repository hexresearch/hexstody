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
        vec![Currency::BTC,
             Currency::ETH,
             Currency::ERC20(Erc20Token{ticker:"USDT".to_string()
                                        ,name:"USDT".to_string()
                                        ,contract:"0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()
                                    }),
             Currency::ERC20(Erc20Token{ticker:"CRV".to_string()
                                        ,name:"CRV".to_string()
                                        ,contract:"0xD533a949740bb3306d119CC777fa900bA034cd52".to_string()
                                    }),
             Currency::ERC20(Erc20Token{ticker:"GTECH".to_string()
                                        ,name:"GTECH".to_string()
                                        ,contract:"0xD533a949740bb3306d119CC777fa900bA034cd52".to_string()
                                    })
            ]
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
pub struct Erc20{
   token : Erc20Token,
   account : EthAccount
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

/// Validated bitcoin address
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct BtcAddress{pub addr: String}

impl fmt::Display for BtcAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.addr)
    }
}

/// Validated ethereum account address
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct EthAccount{pub account: String}

impl fmt::Display for EthAccount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.account)
    }
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct BTCTxid {pub txid: String}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct ETHTxid {pub txid: String}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(tag = "type")]
pub enum CurrencyTxId {
    BTC(BTCTxid),
    ETH(ETHTxid),
}

impl fmt::Display for CurrencyTxId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrencyTxId::BTC(BTCTxid{txid}) => write!(f, "{}", txid),
            CurrencyTxId::ETH(ETHTxid{txid}) => write!(f, "{}", txid),
        }
    }
}

use bitcoin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr, vec};

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

impl FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TUSDT" => Ok(Currency::usdt_erc20()),
            "TCRV" => Ok(Currency::crv_erc20()),
            "TGTECH" => Ok(Currency::gtech_erc20()),
            "ETH" => Ok(Currency::ETH),
            _ => Err(format!("unknown currency{}", s)),
        }
    }
}

impl Currency {
    pub fn usdt_erc20() -> Currency {
        Currency::ERC20(Erc20Token {
            ticker: "USDT".to_string(),
            name: "USDT".to_string(),
            contract: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
        })
    }

    pub fn crv_erc20() -> Currency {
        Currency::ERC20(Erc20Token {
            ticker: "CRV".to_string(),
            name: "CRV".to_string(),
            contract: "0xd533a949740bb3306d119cc777fa900ba034cd52".to_string(),
        })
    }

    pub fn gtech_erc20() -> Currency {
        Currency::ERC20(Erc20Token {
            ticker: "GTECH".to_string(),
            name: "GTECH".to_string(),
            contract: "0x866A4Da32007BA71aA6CcE9FD85454fCF48B140c".to_string(),
        })
    }

    pub fn ticker(&self) -> String {
        self.symbol().symbol()
    }

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

    pub fn get_by_name(name_orig: &str) -> Option<Currency> {
        let name = name_orig.to_uppercase();
        if name == "BTC" {
            return Some(Currency::BTC);
        } else if name == "ETH" {
            return Some(Currency::ETH);
        } else {
            let tokens = Currency::supported_tokens();
            for token in tokens {
                if name == token.ticker {
                    return Some(Currency::ERC20(token));
                };
            }
            return None;
        }
    }

    pub fn symbol(&self) -> Symbol{
        match self {
            Currency::BTC => Symbol::BTC,
            Currency::ETH => Symbol::ETH,
            Currency::ERC20(symbol) => Symbol::ERC20(symbol.ticker.clone()),
        }
    }

    pub fn from_symbol(symbol: Symbol) -> Option<Currency>{
        match symbol {
            Symbol::USD => None,
            Symbol::RUB => None,
            Symbol::BTC => Some(Currency::BTC),
            Symbol::ETH => Some(Currency::ETH),
            Symbol::ERC20(s) => match s.as_str() {
                "USDT" => Some(Currency::usdt_erc20()),
                "CRV" => Some(Currency::crv_erc20()),
                "GTECH" => Some(Currency::gtech_erc20()),
                _ => None
            },
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
    Debug, Serialize, Deserialize, JsonSchema, Clone, Eq, PartialEq, Hash,
)]
pub struct Erc20Token {
    /// Short name of the token like USDT or WBTC
    pub ticker: String,
    /// Long name like 'Wrapped Bitcoin'
    pub name: String,
    /// Contract address
    pub contract: String,
}

impl Erc20Token {
    pub fn index(&self) -> u16 {
        match self.ticker.as_str() {
            "USDT" => 0,
            "GTECH" => 1,
            "CRV" => 2,
            _ => u16::max_value()
        }
    }
}

impl PartialOrd for Erc20Token{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index().partial_cmp(&other.index())
    }
}

impl Ord for Erc20Token {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index().cmp(&other.index())
    }
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
    pub token: Erc20Token,
    pub account: EthAccount,
}

impl CurrencyAddress {
    pub fn currency(&self) -> Currency {
        match self {
            CurrencyAddress::BTC(_) => Currency::BTC,
            CurrencyAddress::ETH(_) => Currency::ETH,
            CurrencyAddress::ERC20(erc20) => Currency::ERC20(erc20.token.clone()),
        }
    }

    pub fn address(&self) -> String{
        match self {
            CurrencyAddress::BTC(v) => v.addr.clone(),
            CurrencyAddress::ETH(v) => v.account.clone(),
            CurrencyAddress::ERC20(v) => v.account.account.clone(),
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

/// Supported fiat currencies. Can be extended in future.
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Fiat {
    USD,
    RUB
}

impl Fiat{
    pub fn symbol(&self) -> Symbol {
        match self {
            Fiat::USD => Symbol::USD ,
            Fiat::RUB => Symbol::RUB ,
        }
    }

    pub fn from_symbol(symbol: Symbol) -> Option<Fiat>{
        match symbol {
            Symbol::USD => Some(Fiat::USD),
            Symbol::RUB => Some(Fiat::RUB),
            _ => None
        }
    }

    pub fn ticker(&self) -> String {
        self.symbol().symbol()
    }
}

/// Generalized tickers. Keep them all together to enable generic storage and request
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Symbol {
    USD,
    RUB,
    BTC,
    ETH,
    ERC20(String)
}

impl Symbol {
    pub fn symbol(&self) -> String {
        match self {
            Symbol::USD => "USD".to_owned(),
            Symbol::RUB => "RUB".to_owned(),
            Symbol::BTC => "BTC".to_owned(),
            Symbol::ETH => "ETH".to_owned(),
            Symbol::ERC20(ticker) => ticker.clone(),
        }
    }

    pub fn is_crypto(&self) -> bool {
        match self {
            Symbol::USD => false,
            Symbol::RUB => false,
            Symbol::BTC => true,
            Symbol::ETH => true,
            Symbol::ERC20(_) => true,
        }
    }

    pub fn is_fiat(&self) -> bool {
        match self {
            Symbol::USD => true,
            Symbol::RUB => true,
            Symbol::BTC => false,
            Symbol::ETH => false,
            Symbol::ERC20(_) => false,
        }
    }

    pub fn supported() -> Vec<Symbol> {
        vec![
            Symbol::USD,
            Symbol::RUB,
            Symbol::BTC,
            Symbol::ETH,
            Symbol::ERC20("USDT".to_owned()),
            Symbol::ERC20("CRV".to_owned()),
            Symbol::ERC20("GTECH".to_owned())
        ]
    }

    pub fn supported_fiats() -> Vec<Symbol> {
        Symbol::supported().iter().filter(|t| t.is_fiat()).cloned().collect()
    }

    pub fn supported_cryptos() -> Vec<Symbol> {
        Symbol::supported().iter().filter(|t| t.is_crypto()).cloned().collect()
    }

    pub fn exponent(&self) -> f64 {
        match self {
            Symbol::USD => 1.0,
            Symbol::RUB => 1.0,
            Symbol::BTC => 100000000.0,
            Symbol::ETH => 1000000000000000000.0,
            Symbol::ERC20(ticker) => match ticker.as_str() {
                "USDT" => 1.0,
                "CRV" => 1.0,
                "GTECH" => 1.0,
                _ => 1.0
            },
        }
    }
}

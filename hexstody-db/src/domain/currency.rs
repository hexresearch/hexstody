use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// A currency that custody understands. Can be extended in future.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Currency {
    BTC,
    ETH,
    ERC20(Erc20Token),
}

/// Description of ERC20 token that allows to distinguish them between each other
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Erc20Token {
    /// Short name of the token like USDT or WBTC
    pub ticker: String,
    /// Long name like 'Wrapped Bitcoin'
    pub name: String,
    /// Contract address
    pub contract: String,
}

/// Address that can be used for receiving currency
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CurrencyAddress {
    BTC(BtcAddress),
    ETH(EthAccount),
    ERC20(Erc20Token, EthAccount),
}

/// Validated bitcoin address
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BtcAddress(pub String);

/// Validated ethereum account address
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EthAccount(pub String);

impl Currency {
    /// List supported currencies at the moment
    pub fn supported() -> Vec<Currency> {
        vec![Currency::BTC, Currency::ETH]
    }
}


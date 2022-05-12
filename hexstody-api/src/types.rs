use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::domain::currency::Currency;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceItem {
    pub currency: Currency,
    pub value: u64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositHistoryItem {
    pub currency: Currency,
    pub value: u64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WithdrawalHistoryItem {
    pub currency: Currency,
    pub value: u64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum HistoryItem {
    Deposit(DepositHistoryItem),
    Withdrawal(WithdrawalHistoryItem),
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Balance {
    pub balances: Vec<BalanceItem>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct History {
    pub history_items: Vec<HistoryItem>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SignupEmail {
    /// Unique email
    pub user: String, 
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SigninEmail {
    pub user: String, 
    pub password: String,
}
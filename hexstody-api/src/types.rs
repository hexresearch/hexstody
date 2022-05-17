use chrono::NaiveDateTime;

use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::domain::currency::Currency;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum WithdrawalRequestStatus {
    UnderReview,
    InProgress,
    AwaitsApproval,
    Completed,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceItem {
    pub currency: Currency,
    pub value: i64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositHistoryItem {
    pub currency: Currency,
    pub date: NaiveDateTime,
    pub value: u64,
    pub number_of_confirmations: u64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WithdrawalHistoryItem {
    pub currency: Currency,
    pub date: NaiveDateTime,
    pub value: u64,
    pub status: WithdrawalRequestStatus,
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

impl Balance {
    pub fn by_currency(&self, curr: &Currency) -> Option<&BalanceItem> {
        self.balances.iter().find(|i| i.currency == *curr)
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct History {
    pub target_number_of_confirmations: u64,
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositInfo {
    pub address: String,
}

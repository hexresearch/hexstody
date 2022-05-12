use chrono::NaiveDateTime;
use rocket::serde::json::Json;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use hexstody_db::domain::currency::Currency;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceItem {
    pub currency: Currency,
    pub value: u64,
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
    pub target_number_of_confirmations: u64,
    pub history_items: Vec<HistoryItem>
}

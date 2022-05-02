use rocket::serde::json::Json;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use hexstody_db::domain::currency::{Currency};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceItem {
    pub currency : Currency,
    pub value : u64
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HistoryItem {
    pub is_deposit : bool, 
    pub currency : Currency,
    pub value : u64
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Balance {
    pub balances: Vec<BalanceItem>
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct History {
    pub history_items: Vec<HistoryItem>
}
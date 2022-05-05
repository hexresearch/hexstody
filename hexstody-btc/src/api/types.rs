use serde::{Deserialize, Serialize};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositEvents {
    pub events: Vec<DepositEvent>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositEvent {
    pub address: String,
    /// Sats amount
    pub amount: u64,
    /// 0 means unconfirmed
    pub confirmations: u64,
    /// UNIX timestamp when the event occured
    pub timestamp: i64, 
}

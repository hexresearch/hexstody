use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::{JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositEvents {
    pub events: Vec<DepositEvent>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum DepositEvent {
    New(DepositTxUpdate),
    Cancel(DepositTxCancel),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositTxUpdate {
    /// Transaction ID (txid)
    pub txid: String,
    /// Which output of the transaction
    pub vout: u32,
    /// Address that tx tops up
    pub address: String,
    /// Sats amount
    pub amount: u64,
    /// 0 means unconfirmed
    pub confirmations: u64,
    /// UNIX timestamp when the event occured
    pub timestamp: i64,
}

/// Unconfirmed tx cancel or even reorg cancel
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositTxCancel {
    /// Transaction ID (txid)
    pub txid: String,
    /// Which output of the transaction
    pub vout: u32,
    /// Address that tx tops up
    pub address: String,
    /// Sats amount
    pub amount: u64,
    /// UNIX timestamp when the event occured
    pub timestamp: i64,
}


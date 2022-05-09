use super::bitcoin::*;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DepositEvents {
    /// New block height
    pub height: u64,
    /// Hash of block
    pub hash: BtcBlockHash,
    /// New updates on transactions in that block
    pub events: Vec<DepositEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum DepositEvent {
    Update(DepositTxUpdate),
    Cancel(DepositTxCancel),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DepositTxUpdate {
    /// Transaction ID (txid)
    pub txid: BtcTxid,
    /// Which output of the transaction
    pub vout: u32,
    /// Address that tx tops up
    pub address: BtcAddress,
    /// Sats amount
    pub amount: u64,
    /// 0 means unconfirmed
    pub confirmations: u64,
    /// UNIX timestamp when the event occured
    pub timestamp: u64,
}

/// Unconfirmed tx cancel or even reorg cancel
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    pub timestamp: u64,
}

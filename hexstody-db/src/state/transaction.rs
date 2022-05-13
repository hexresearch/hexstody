use bitcoin::{Txid, Address};
use serde::{Deserialize, Serialize};
use chrono::prelude::*;

/// User data for specific currency
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Transaction {
    Btc(BtcTransaction),
    Eth(),
}

/// Bitcoin transaction metainformation
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BtcTransaction {
    /// ID of transaction
    pub txid: Txid,
    /// Output of transaction that belongs to us
    pub vout: u32,
    /// Top up address
    pub address: Address,
    /// 0 means unconfirmed
    pub confirmations: u64,
    /// Negative for withdrawal, positive for deposit
    pub amount: i64,
    /// The tx first seen 
    pub timestamp: NaiveDateTime,
    /// Conflicts with other transactions
    pub conflicts: Vec<Txid>,
}
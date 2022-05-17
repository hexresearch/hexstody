use bitcoin::{Address, Txid};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

/// User data for specific currency
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Transaction {
    Btc(BtcTransaction),
    Eth(),
}

impl Transaction {
    pub fn amount(&self) -> i64 {
        match self {
            Transaction::Btc(tx) => tx.amount,
            Transaction::Eth() => 0,
        }
    }

    pub fn is_finalized(&self) -> bool {
        match self {
            Transaction::Btc(tx) => tx.confirmations > 3,
            Transaction::Eth() => todo!("Eth confirmations"),
        }
    }

    pub fn is_withdraw(&self) -> bool {
        match self {
            Transaction::Btc(tx) => tx.amount < 0,
            Transaction::Eth() => todo!("Eth is withdraw"),
        }
    }
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

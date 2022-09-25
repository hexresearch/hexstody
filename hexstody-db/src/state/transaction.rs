use crate::update::btc::BtcTxCancel;
use bitcoin::{Address, Txid};
use chrono::prelude::*;
use hexstody_btc_api::events::{TxDirection, TxUpdate};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// User data for specific currency
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Transaction {
    Btc(BtcTransaction),
    Eth(EthTransaction),
}

impl Transaction {
    pub fn amount(&self) -> i64 {
        match self {
            Transaction::Btc(tx) => tx.amount,
            Transaction::Eth(tx) => tx.value,
        }
    }

    pub fn is_finalized(&self) -> bool {
        match self {
            Transaction::Btc(tx) => tx.confirmations > 3,
            Transaction::Eth(tx) => tx.confirmations > 3,
        }
    }

    pub fn is_withdraw(&self) -> bool {
        match self {
            Transaction::Btc(tx) => tx.amount < 0,
            Transaction::Eth(tx) => tx.from == tx.addr,
        }
    }

    pub fn is_conflicted(&self) -> bool {
        match self {
            Transaction::Btc(tx) => tx.confirmations == 0 && !tx.conflicts.is_empty(),
            Transaction::Eth(tx) => tx.confirmations < 0,
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
    /// Fee paid in sats.
    /// Only available for outgoing transactions.
    pub fee: Option<u64>,
}

pub trait SameBtcTx<T> {
    /// Check that the tx is describes the same record in the blockchain
    fn is_same_btc_tx(&self, other: &T) -> bool;
}

impl SameBtcTx<BtcTransaction> for BtcTransaction {
    fn is_same_btc_tx(&self, other: &BtcTransaction) -> bool {
        self.txid == other.txid && self.vout == other.vout
    }
}

impl SameBtcTx<BtcTxCancel> for BtcTransaction {
    fn is_same_btc_tx(&self, other: &BtcTxCancel) -> bool {
        self.txid == other.txid.0 && self.vout == other.vout
    }
}

impl From<TxUpdate> for BtcTransaction {
    fn from(val: TxUpdate) -> BtcTransaction {
        BtcTransaction {
            txid: val.txid.0,
            vout: val.vout,
            address: val.address.0,
            confirmations: val.confirmations,
            amount: match val.direction {
                TxDirection::Deposit => val.amount as i64,
                TxDirection::Withdraw => -(val.amount as i64),
            },
            timestamp: NaiveDateTime::from_timestamp(val.timestamp as i64, 0),
            conflicts: val.conflicts.iter().map(|tx| tx.0).collect(),
            fee: val.fee
        }
    }
}

/// Ethereum transaction metainformation
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EthTransaction {
    /// The number of block
    pub blockNumber: String,
    /// The tx first seen
    pub timeStamp: String,
    /// ID of transaction
    pub hash: String,
    /// Sender's address
    pub from: String,
    /// Recipient's address
    pub to: String,
    /// Amount of money
    pub value: i64,
    /// Name of sending token
    pub tokenName: String,
    /// Gas spent for this transaction
    pub gas: String,
    /// Gas price at the moment of transaction execution
    pub gasPrice: String,
    /// Address of token's contract
    pub contractAddress: String,
    /// Confirmations number
    pub confirmations: i64,
    /// Accounts address
    pub addr: String
}

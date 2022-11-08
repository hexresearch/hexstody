use super::ethereum::*;
use web3::types::H256;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EthEvents {
    /// New block height
    pub height: u64,
    /// Hash of block
    pub hash: EthTxid,
    /// New updates on transactions in that block
    pub events: Vec<EthEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum EthEvent {
    Update(TxUpdate),
    Cancel(TxCancel),
}

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Ord, PartialOrd, Hash,
)]
pub enum TxDirection {
    Deposit,
    Withdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TxUpdate {
    /// Direction of tx (in or out)
    pub direction: TxDirection,
    /// Transaction ID (txid)
    pub txid: EthTxid,
    /// Which output of the transaction
    pub vout: u32,
    /// Address that tx tops up
    pub address: EthAddress,
    /// Sats amount
    pub amount: u64,
    /// 0 means unconfirmed
    pub confirmations: u64,
    /// UNIX timestamp when the event occured
    pub timestamp: u64,
    /// Other transaction that are in conflict with the tx
    /// That means that they are RBF transactions and one
    /// eventually will replace the others.
    pub conflicts: Vec<EthTxid>,
    /// Fee paid in sats.
    /// Only available for outgoing transactions.
    pub fee: Option<u64>,
}

/// Unconfirmed tx cancel or even reorg cancel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TxCancel {
    /// Direction of tx (in or out)
    pub direction: TxDirection,
    /// Transaction ID (txid)
    pub txid: EthTxid,
    /// Which output of the transaction
    pub vout: u32,
    /// Address that tx tops up
    pub address: EthAddress,
    /// Sats amount
    pub amount: u64,
    /// UNIX timestamp when the event occured
    pub timestamp: u64,
    /// Other transaction that are in conflict with the tx
    /// That means that they are RBF transactions and one
    /// eventually will replace the others.
    pub conflicts: Vec<EthTxid>,
}

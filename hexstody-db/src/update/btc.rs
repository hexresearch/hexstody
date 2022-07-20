use hexstody_btc_api::events::TxCancel;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BestBtcBlock {
    pub height: u64,
    pub block_hash: String,
}

pub type BtcTxCancel = TxCancel;

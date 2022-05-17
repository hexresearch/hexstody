use serde::{Serialize, Deserialize};
use hexstody_btc_api::events::TxCancel;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BestBtcBlock {
    pub height: u64,
    pub block_hash: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BtcTxCancel {
    pub txid: String, 
    pub vout: u32,
    pub address: String,
}

impl From<TxCancel> for BtcTxCancel {
    fn from(val: TxCancel) -> BtcTxCancel {
        BtcTxCancel {
            txid: val.txid.0.to_string(),
            vout: val.vout,
            address: val.address.0.to_string(),
        }
    }
}
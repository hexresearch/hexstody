use bitcoin::blockdata::constants::genesis_block;
use bitcoin::network::constants::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BtcState {
    pub height: u64,
    pub block_hash: String,
}

impl BtcState {
    pub fn new(network: Network) -> Self {
        BtcState {
            height: 0,
            block_hash: genesis_block(network).block_hash().to_string(),
        }
    }
}

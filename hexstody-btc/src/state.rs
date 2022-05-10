use bitcoin::blockdata::constants::genesis_block;
use bitcoin::hash_types::BlockHash;
use bitcoin::network::constants::Network;
use hexstody_btc_api::events::*;

pub struct ScanState {
    pub last_block: BlockHash,
    pub last_height: u64,
    pub network: Network,
    pub events: Vec<BtcEvent>,
}

impl Default for ScanState {
    fn default() -> Self {
        ScanState::new(Network::Bitcoin)
    }
}

impl ScanState {
    pub fn new(network: Network) -> Self {
        ScanState {
            last_block: genesis_block(network).block_hash(),
            last_height: 0,
            network,
            events: vec![],
        }
    }
}

use bitcoin::blockdata::constants::genesis_block;
use bitcoin::hash_types::BlockHash;
use bitcoin::network::constants::Network;
use hexstody_btc_api::deposit::*;

pub struct ScanState {
    pub last_block: BlockHash,
    pub network: Network,
    pub deposit_events: Vec<DepositEvent>,
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
            network,
            deposit_events: vec![],
        }
    }
}

use bitcoin::hash_types::BlockHash;
use bitcoin::blockdata::constants::genesis_block;
use super::api::types::deposit::*;
use bitcoin::network::constants::Network;

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
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::hash_types::BlockHash;
use bitcoin::network::constants::Network;
use hexstody_btc_api::events::*;
use hexstody_eth_api::events::*;
use web3::types::{BlockId, BlockNumber};

pub struct ScanState {
    pub last_block_eth: BlockNumber,
    pub last_block: BlockHash,
    pub last_height: u64,
    pub network: Network,
    pub events: Vec<BtcEvent>,
    pub events_eth: Vec<EthEvent>
}

impl Default for ScanState {
    fn default() -> Self {
        ScanState::new(Network::Bitcoin)
    }
}

impl ScanState {
    pub fn new(network: Network) -> Self {
        ScanState {
            last_block_eth: BlockNumber::Earliest,
            last_block: genesis_block(network).block_hash(),
            last_height: 0,
            network,
            events: vec![],
            events_eth: vec![],
        }
    }
}

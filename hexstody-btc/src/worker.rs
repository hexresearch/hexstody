use crate::state::ScanState;
use bitcoin::{hash_types::BlockHash, Address, Amount};
use bitcoincore_rpc::{Client, RpcApi};
use bitcoincore_rpc_json::{GetTransactionResultDetailCategory, ListTransactionResult};
use hexstody_btc_api::events::*;
use log::*;
use std::{sync::Arc};
use std::time::Duration;
use tokio::sync::{Mutex, Notify};

pub async fn node_worker(
    client: &Client,
    state: Arc<Mutex<ScanState>>,
    state_notify: Arc<Notify>,
    polling_sleep: Duration,
    tx_notify: Arc<Notify>
) -> () {
    loop {
        {
            let mut state_rw = state.lock().await;
            let old_block = state_rw.last_block;
            match scan_from(client, old_block).await {
                Ok((mut events, next_hash)) => {
                    if !events.is_empty() || old_block != next_hash {
                        let height = client
                            .get_block_count()
                            .unwrap_or_else(|_| state_rw.last_height);
                        state_rw.last_height = height;
                        state_rw.last_block = next_hash;
                        if !events.is_empty() {
                            info!("New events {}", events.len());
                            state_rw.events.append(&mut events);
                        }
                        state_notify.notify_one();
                        tx_notify.notify_one();
                    }
                }
                Err(e) => {
                    error!("Failed to query node: {e}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
        tokio::time::sleep(polling_sleep).await;
    }
}

pub async fn cold_wallet_worker(
    client: &Client,
    tx_notify: Arc<Notify>,
    cold_amount: Amount,
    cold_address: Address
) -> (){
    loop {
        tx_notify.notified().await;
        if let Ok(bal) = client.get_balance(Some(3), None){
            if bal > cold_amount {
                let amount = if bal > (cold_amount + cold_amount) { bal - cold_amount } else {cold_amount};
                match client.send_to_address(&cold_address, amount, None, None, Some(true), None, None, None){
                    Ok(txid) => info!("Dumped {} to cold wallet. Txid: {}", amount, txid),
                    Err(e) => error!("Failed to dump to cold wallet. {}", e),
                }
            }
        }
    }    
}

pub async fn scan_from(
    client: &Client,
    blockhash: BlockHash,
) -> bitcoincore_rpc::Result<(Vec<BtcEvent>, BlockHash)> {
    let result = client.list_since_block(Some(&blockhash), None, Some(false), Some(true))?;
    let mut events = vec![];
    for tx in result.removed {
        if let Some(e) = to_remove_event(tx) {
            events.push(e);
        }
    }
    for tx in result.transactions {
        if let Some(e) = to_update_event(tx) {
            events.push(e);
        }
    }
    Ok((events, result.lastblock))
}

fn tx_direction(tx: &ListTransactionResult) -> Option<TxDirection> {
    match tx.detail.category {
        GetTransactionResultDetailCategory::Receive => {
            info!("Found new incoming transaction {:?}", tx.info.txid);
            debug!("Info: {:?}", tx.info);
            debug!("Details: {:?}", tx.detail);
            Some(TxDirection::Deposit)
        }
        GetTransactionResultDetailCategory::Send => {
            info!("Found new outcoming transaction {:?}", tx.info.txid);
            debug!("Info: {:?}", tx.info);
            debug!("Details: {:?}", tx.detail);
            Some(TxDirection::Withdraw)
        }
        _ => {
            info!(
                "The tx {:?} has wrong type {:?}",
                tx.info.txid, tx.detail.category
            );
            None
        }
    }
}

fn to_update_event(tx: ListTransactionResult) -> Option<BtcEvent> {
    let direction = if let Some(dir) = tx_direction(&tx) {
        dir
    } else {
        return None;
    };

    let address = if let Some(address) = tx.detail.address {
        address.into()
    } else {
        warn!("Transaction {:?} doesn't have address", tx.info.txid);
        return None;
    };

    if tx.info.confirmations < 0 {
        warn!(
            "Transaction {:?} has negative amount of confirmations {:?}",
            tx.info.txid, tx.info.confirmations
        );
        return None;
    }

    Some(BtcEvent::Update(TxUpdate {
        direction,
        txid: tx.info.txid.into(),
        vout: tx.detail.vout,
        address,
        amount: tx.detail.amount.as_sat().abs() as u64,
        confirmations: tx.info.confirmations as u64,
        timestamp: tx.info.timereceived,
        conflicts: tx
            .info
            .wallet_conflicts
            .into_iter()
            .map(|v| v.into())
            .collect(),
    }))
}

fn to_remove_event(tx: ListTransactionResult) -> Option<BtcEvent> {
    let direction = if let Some(dir) = tx_direction(&tx) {
        dir
    } else {
        return None;
    };

    let address = if let Some(address) = tx.detail.address {
        address.into()
    } else {
        warn!("Transaction {:?} doesn't have address", tx.info.txid);
        return None;
    };

    if tx.detail.amount.as_sat() < 0 {
        warn!(
            "Transaction {:?} has negative amount of sats {:?}",
            tx.info.txid, tx.detail.amount
        );
        return None;
    }

    Some(BtcEvent::Cancel(TxCancel {
        direction,
        txid: tx.info.txid.into(),
        vout: tx.detail.vout,
        address,
        amount: tx.detail.amount.as_sat().abs() as u64,
        timestamp: tx.info.timereceived,
        conflicts: tx
            .info
            .wallet_conflicts
            .into_iter()
            .map(|v| v.into())
            .collect(),
    }))
}

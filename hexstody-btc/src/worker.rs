use crate::state::ScanState;
use bitcoin::hash_types::BlockHash;
use bitcoincore_rpc::{Client, RpcApi};
use bitcoincore_rpc_json::{GetTransactionResultDetailCategory, ListTransactionResult};
use hexstody_btc_api::deposit::*;
use log::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};

pub async fn node_worker(
    client: &Client,
    state: Arc<Mutex<ScanState>>,
    state_notify: Arc<Notify>,
    polling_sleep: Duration,
) -> () {
    loop {
        {
            let mut state_rw = state.lock().await;
            match scan_from(client, state_rw.last_block).await {
                Ok((mut events, next_hash)) => {
                    state_rw.last_block = next_hash;
                    if !events.is_empty() {
                        info!("New events {}", events.len());
                        state_rw.deposit_events.append(&mut events);
                        state_notify.notify_one();
                    }
                }
                Err(e) => {
                    error!("Failed to query node: {e}");
                }
            }
        }
        tokio::time::sleep(polling_sleep).await;
    }
}

pub async fn scan_from(
    client: &Client,
    blockhash: BlockHash,
) -> bitcoincore_rpc::Result<(Vec<DepositEvent>, BlockHash)> {
    let result = client.list_since_block(Some(&blockhash), None, Some(false), Some(true))?;
    let mut events = vec![];
    for tx in result.transactions {
        if let Some(e) = to_deposit_update_event(tx) {
            events.push(e);
        }
    }
    for tx in result.removed {
        if let Some(e) = to_deposit_remove_event(tx) {
            events.push(e);
        }
    }
    Ok((events, result.lastblock))
}

fn to_deposit_update_event(tx: ListTransactionResult) -> Option<DepositEvent> {
    if let GetTransactionResultDetailCategory::Receive = tx.detail.category {
        info!("Found new incoming transaction {:?}", tx.info.txid);
    } else {
        info!(
            "The tx {:?} has wrong type {:?}",
            tx.info.txid, tx.detail.category
        );
        return None;
    }

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
    if tx.info.confirmations < 0 {
        warn!(
            "Transaction {:?} has negative amount of confirmations {:?}",
            tx.info.txid, tx.info.confirmations
        );
        return None;
    }

    Some(DepositEvent::Update(DepositTxUpdate {
        txid: tx.info.txid.into(),
        vout: tx.detail.vout,
        address,
        amount: tx.detail.amount.as_sat() as u64,
        confirmations: tx.info.confirmations as u64,
        timestamp: tx.info.timereceived,
    }))
}

fn to_deposit_remove_event(tx: ListTransactionResult) -> Option<DepositEvent> {
    if let GetTransactionResultDetailCategory::Receive = tx.detail.category {
        info!("Found new canceled deposit transaction {:?}", tx.info.txid);
    } else {
        info!(
            "The tx {:?} has wrong type {:?}",
            tx.info.txid, tx.detail.category
        );
        return None;
    }

    let address = if let Some(address) = tx.detail.address {
        address.to_string()
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

    Some(DepositEvent::Cancel(DepositTxCancel {
        txid: tx.info.txid.to_string(),
        vout: tx.detail.vout,
        address,
        amount: tx.detail.amount.as_sat() as u64,
        timestamp: tx.info.timereceived,
    }))
}

use hexstody_btc_api::events::*;
use hexstody_btc_client::client::BtcClient;
use hexstody_db::{
    state::State,
    update::{btc::BestBtcBlock, StateUpdate, UpdateBody},
};
use log::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;

pub async fn btc_worker(
    btc_client: BtcClient,
    state_mx: Arc<Mutex<State>>,
    update_sender: mpsc::Sender<StateUpdate>,
) {
    trace!("Starting BTC worker");
    loop {
        match btc_client.poll_events().await {
            Ok(events) => {
                process_btc_events(state_mx.clone(), &update_sender, events).await;
            }
            Err(e) => {
                error!("BTC module error: {e}");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

pub async fn process_btc_events(
    state_mx: Arc<Mutex<State>>,
    update_sender: &mpsc::Sender<StateUpdate>,
    events: BtcEvents,
) {
    let block_hash = events.hash.0.to_string();
    {
        let state = state_mx.lock().await;
        if state.btc_state.block_hash != block_hash {
            update_sender
                .send(StateUpdate::new(UpdateBody::BestBtcBlock(BestBtcBlock {
                    height: events.height,
                    block_hash,
                })))
                .await
                .unwrap();
        }

        for event in events.events {
            match event {
                BtcEvent::Update(upd) => {
                    if upd.direction == TxDirection::Deposit {
                        update_sender
                            .send(StateUpdate::new(UpdateBody::UpdateBtcTx(upd.into())))
                            .await
                            .unwrap();
                    }
                }
                BtcEvent::Cancel(cnl) => {
                    update_sender
                        .send(StateUpdate::new(UpdateBody::CancelBtcTx(cnl.into())))
                        .await
                        .unwrap();
                }
            }
        }
    }
}

use hexstody_api::types::ConfirmedWithdrawal;
use hexstody_btc_api::events::*;
use hexstody_btc_client::client::BtcClient;
use hexstody_db::{
    state::State,
    update::{btc::BestBtcBlock, StateUpdate, UpdateBody, results::UpdateResult},
};
use log::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;

pub async fn update_results_worker(
    btc_client: BtcClient,
    state_mx: Arc<Mutex<State>>,
    mut update_receiver: mpsc::Receiver<UpdateResult>,
){
    trace!("Starting update results worker");
    loop {
        match update_receiver.recv().await {
            Some(upd) => match upd {
                UpdateResult::WithdrawConfirmed(id) => {
                    let sreq = {   
                        let state = state_mx.lock().await;
                        state.get_withdrawal_request(id)
                    };
                    if let Some(req) = sreq {
                        let confirmations = vec![];
                        let rejections = vec![];
                        let cw = ConfirmedWithdrawal{
                            id,
                            user: req.user,
                            address: req.address,
                            created_at: req.created_at.to_string(),
                            amount: req.amount,
                            confirmations,
                            rejections,
                        };
                        if let Err(e) = btc_client.withdraw_btc(cw).await{
                            error!("Failed to post tx: {:?}", e);
                        }
                    }
                }

            },
            None => break,
        }
    }

}

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

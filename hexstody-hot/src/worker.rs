use hexstody_btc_api::events::*;
use hexstody_btc_client::client::BtcClient;
use hexstody_db::{state::State, update::StateUpdate};
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
                process_btc_events(&btc_client, state_mx.clone(), &update_sender, events).await;
            }
            Err(e) => {
                error!("BTC module error: {e}");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub async fn process_btc_events(
    btc_client: &BtcClient,
    state_mx: Arc<Mutex<State>>,
    update_sender: &mpsc::Sender<StateUpdate>,
    events: BtcEvents,
) {
}

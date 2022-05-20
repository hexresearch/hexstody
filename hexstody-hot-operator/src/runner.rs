use log::*;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, Notify};

use hexstody_db::queries::query_state;
use hexstody_db::state::Network;
use hexstody_db::*;

use super::api::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Database query error: {0}")]
    Query(#[from] hexstody_db::queries::Error),
    #[error("Rocket error: {0}")]
    RocketError(rocket::Error),
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

pub async fn run_api(
    network: Network,
    db_connect: &str,
    start_notify: Arc<Notify>,
) -> Result<(), Error> {
    info!("Connecting to database");
    let pool = create_db_pool(db_connect).await?;
    info!("Reconstructing state from database");
    let state = query_state(network, &pool).await?;
    let state_mx = Arc::new(Mutex::new(state));
    let state_notify = Arc::new(Notify::new());
    let (update_sender, update_receiver) = mpsc::channel(1000);

    let update_worker_handle = tokio::spawn({
        let pool = pool.clone();
        let state_mx = state_mx.clone();
        let state_notify = state_notify.clone();
        async move {
            update_worker(pool, state_mx, state_notify, update_receiver).await;
        }
    });

    let api_handle = tokio::spawn(serve_api(
        pool,
        state_mx,
        state_notify,
        start_notify,
        update_sender,
    ));
    // Update worker finishes automatically when api worker closes
    // as there is no more active channel senders.
    let (_, api_res) = tokio::join!(update_worker_handle, api_handle);
    api_res?.map_err(|err| Error::RocketError(err))
}

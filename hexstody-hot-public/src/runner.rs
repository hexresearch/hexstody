use futures::future::{join_all, AbortHandle, AbortRegistration, Abortable, Aborted};
use futures::Future;
use log::*;
use serde::Deserialize;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, Notify};

use hexstody_btc_client::client::BtcClient;
use hexstody_db::queries::query_state;
use hexstody_db::*;
use hexstody_db::{
    state::{Network, State},
    update::StateUpdate,
    Pool,
};

use super::api::public::*;
use super::worker::*;

async fn serve_abortable<F, Fut, Out>(abort_reg: AbortRegistration, api_future: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Out> + Send + 'static,
    Out: Send + 'static,
{
    let abortable_api_futute = tokio::spawn(Abortable::new(api_future(), abort_reg));
    match abortable_api_futute.await {
        Ok(Err(Aborted)) => {
            error!("API thread aborted");
            return ();
        }
        Ok(_) => (),
        Err(error) => error!("{:?}", error),
    };
}

pub async fn serve_apis(
    pool: Pool,
    state_mx: Arc<Mutex<State>>,
    state_notify: Arc<Notify>,
    start_notify: Arc<Notify>,
    api_abort: AbortRegistration,
    update_sender: mpsc::Sender<StateUpdate>,
    btc_client: BtcClient,
) -> Result<(), Aborted> {
    let (api_handle, abort_reg) = AbortHandle::new_pair();
    let api_fut = serve_abortable(abort_reg, || {
        serve_api(
            pool.clone(),
            state_mx.clone(),
            state_notify.clone(),
            start_notify.clone(),
            update_sender.clone(),
            btc_client.clone(),
        )
    });
    let abortable_api = Abortable::new(api_fut, api_abort);
    if let Err(Aborted) = abortable_api.await {
        api_handle.abort();
        info!("API aborted!");
        return Err(Aborted);
    } else {
        return Ok(());
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Database query error: {0}")]
    Query(#[from] hexstody_db::queries::Error),
    #[error("API was aborted from outside")]
    Aborted,
}

pub async fn run_api(
    network: Network,
    db_connect: &str,
    start_notify: Arc<Notify>,
    btc_client: BtcClient,
    api_abort_reg: AbortRegistration,
) -> Result<(), Error> {
    info!("Connecting to database");
    let pool = create_db_pool(db_connect).await?;
    info!("Reconstructing state from database");
    let state = query_state(network, &pool).await?;
    let state_mx = Arc::new(Mutex::new(state));
    let state_notify = Arc::new(Notify::new());
    let (update_sender, update_receiver) = mpsc::channel(1000);

    let update_worker_hndl = tokio::spawn({
        let pool = pool.clone();
        let state_mx = state_mx.clone();
        let state_notify = state_notify.clone();
        async move {
            update_worker(pool, state_mx, state_notify, update_receiver).await;
        }
    });
    let btc_worker_hndl = tokio::spawn({
        let state_mx = state_mx.clone();
        let btc_client = btc_client.clone();
        let update_sender = update_sender.clone();
        async move {
            btc_worker(btc_client, state_mx, update_sender).await;
        }
    });

    if let Err(Aborted) = serve_apis(
        pool,
        state_mx,
        state_notify,
        start_notify,
        api_abort_reg,
        update_sender,
        btc_client,
    )
    .await
    {
        info!("Logic aborted, exiting...");
        update_worker_hndl.abort();
        btc_worker_hndl.abort();
        Err(Error::Aborted)
    } else {
        Ok(())
    }
}

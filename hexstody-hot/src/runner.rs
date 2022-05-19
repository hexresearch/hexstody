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

use super::api::operator::*;
use super::api::public::*;
use super::worker::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApiType {
    Public,
    Operator,
}

impl fmt::Display for ApiType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ApiType::Public => write!(f, "Public"),
            ApiType::Operator => write!(f, "Operator"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub public_api_enabled: bool,
    pub public_api_port: u16,
    pub operator_api_enabled: bool,
    pub operator_api_port: u16,
}

impl ApiConfig {
    pub fn parse_figment() -> Self {
        let figment = rocket::Config::figment();
        let public_api_enabled = figment.extract_inner("public_api_enabled").unwrap_or(true);
        let public_api_port = figment.extract_inner("public_api_port").unwrap_or(9800);
        let operator_api_enabled = figment
            .extract_inner("operator_api_enabled")
            .unwrap_or(true);
        let operator_api_port = figment.extract_inner("operator_api_port").unwrap_or(9801);
        ApiConfig {
            public_api_enabled,
            public_api_port,
            operator_api_enabled,
            operator_api_port,
        }
    }
}

async fn serve_abortable<F, Fut, Out>(
    api_type: ApiType,
    abort_reg: AbortRegistration,
    api_future: F,
) where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Out> + Send + 'static,
    Out: Send + 'static,
{
    let abortable_api_futute = tokio::spawn(Abortable::new(api_future(), abort_reg));
    match abortable_api_futute.await {
        Ok(Err(Aborted)) => {
            error!("{api_type} API thread aborted");
            return ();
        }
        Ok(_) => (),
        Err(error) => error!("{:?}", error),
    };
}

async fn serve_api(
    pool: Pool,
    state_mx: Arc<Mutex<State>>,
    state_notify: Arc<Notify>,
    start_notify: Arc<Notify>,
    api_type: ApiType,
    api_enabled: bool,
    port: u16,
    abort_reg: AbortRegistration,
    update_sender: mpsc::Sender<StateUpdate>,
    btc_client: BtcClient,
    secret_key: Option<String>,
    static_path: String,
) -> () {
    if !api_enabled {
        info!("{api_type} API disabled");
        return ();
    };
    info!("Serving {api_type} API");
    match api_type {
        ApiType::Public => {
            serve_abortable(api_type, abort_reg, || {
                serve_public_api(
                    pool.clone(),
                    state_mx.clone(),
                    state_notify.clone(),
                    start_notify.clone(),
                    port,
                    update_sender.clone(),
                    btc_client,
                    secret_key,
                    static_path,
                )
            })
            .await;
        }
        ApiType::Operator => {
            serve_abortable(api_type, abort_reg, || {
                serve_operator_api(
                    pool.clone(),
                    state_mx.clone(),
                    state_notify.clone(),
                    start_notify.clone(),
                    port,
                    update_sender.clone(),
                    secret_key,
                    static_path,
                )
            })
            .await;
        }
    };
}

pub async fn serve_apis(
    pool: Pool,
    state_mx: Arc<Mutex<State>>,
    state_notify: Arc<Notify>,
    start_notify: Arc<Notify>,
    api_config: ApiConfig,
    api_abort: AbortRegistration,
    update_sender: mpsc::Sender<StateUpdate>,
    btc_client: BtcClient,
    secret_key: Option<&str>,
    static_path: &str,
) -> Result<(), Aborted> {
    let (public_handle, public_abort) = AbortHandle::new_pair();
    let public_api_fut = serve_api(
        pool.clone(),
        state_mx.clone(),
        state_notify.clone(),
        start_notify.clone(),
        ApiType::Public,
        api_config.public_api_enabled,
        api_config.public_api_port,
        public_abort,
        update_sender.clone(),
        btc_client.clone(),
        secret_key.map(|s| s.to_owned()),
        static_path.to_owned(),
    );
    let (operator_handle, operator_abort) = AbortHandle::new_pair();
    let operator_api_fut = serve_api(
        pool,
        state_mx,
        state_notify,
        start_notify,
        ApiType::Operator,
        api_config.operator_api_enabled,
        api_config.operator_api_port,
        operator_abort,
        update_sender.clone(),
        btc_client,
        secret_key.map(|s| s.to_owned()),
        static_path.to_owned(),
    );

    let abortable_apis =
        Abortable::new(join_all(vec![public_api_fut, operator_api_fut]), api_abort);
    if let Err(Aborted) = abortable_apis.await {
        public_handle.abort();
        operator_handle.abort();
        info!("All APIs are aborted!");
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
    #[error("Hot wallet was aborted from outside")]
    Aborted,
}

pub async fn run_hot_wallet(
    network: Network,
    api_config: ApiConfig,
    db_connect: &str,
    start_notify: Arc<Notify>,
    btc_client: BtcClient,
    api_abort_reg: AbortRegistration,
    secret_key: Option<&str>,
    static_path: &str,
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
        api_config,
        api_abort_reg,
        update_sender,
        btc_client,
        secret_key,
        static_path,
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

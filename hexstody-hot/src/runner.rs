use figment::Figment;
use futures::future::{join3, AbortHandle, AbortRegistration, Abortable, Aborted};
use futures::Future;
use hexstody_eth_client::client::EthClient;
use hexstody_runtime_db::RuntimeState;
use hexstody_ticker::worker::ticker_worker;
use hexstody_ticker_provider::client::TickerClient;
use log::*;
use p256::pkcs8::DecodePublicKey;
use p256::PublicKey;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fmt, fs};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, Notify};

use hexstody_btc_client::client::BtcClient;
use hexstody_db::queries::query_state;
use hexstody_db::*;
use hexstody_db::{state::State, update::StateUpdate, Pool};
use hexstody_operator;
use hexstody_public;

use super::worker::*;
use super::Args;

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

#[derive(Debug)]
pub struct ApiConfig {
    pub public_api_config: Figment,
    pub operator_api_config: Figment,
}

impl ApiConfig {
    pub fn parse_figment(args: &Args) -> Self {
        let figment = rocket::Config::figment();
        let default_secret_key =
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".to_owned();
        let mut operator_public_keys = vec![]; //args.operator_public_keys.clone()
        for p in &args.operator_public_keys {
            let full_path = fs::canonicalize(&p).expect("Something went wrong reading the file");
            let key_str =
                fs::read_to_string(full_path).expect("Something went wrong reading the file");
            let public_key = PublicKey::from_public_key_pem(&key_str)
                .expect("Something went wrong decoding the key file");
            operator_public_keys.push(public_key);
        }
        let public_api_enabled = if args.public_api_enabled {
            true
        } else {
            figment.extract_inner("public_api_enabled").unwrap_or(true)
        };
        let public_api_domain = args.public_api_domain.clone().unwrap_or(
            figment
                .extract_inner("public_api_domain")
                .unwrap_or("http://127.0.0.1:9800".to_owned()),
        );
        let public_api_port = args
            .public_api_port
            .unwrap_or(figment.extract_inner("public_api_port").unwrap_or(9800));
        let public_api_static_path = args.public_api_static_path.clone().unwrap_or(
            figment
                .extract_inner("public_api_static_path")
                .unwrap_or(PathBuf::from(rocket::fs::relative!(
                    "../hexstody-public/static/"
                ))),
        );
        let public_api_template_path = args.public_api_template_path.clone().unwrap_or(
            figment
                .extract_inner("public_api_template_path")
                .unwrap_or(PathBuf::from(rocket::fs::relative!(
                    "../hexstody-public/templates/"
                ))),
        );
        let public_api_secret_key = args.public_api_secret_key.clone().unwrap_or(
            figment
                .extract_inner("public_api_secret_key")
                .unwrap_or(default_secret_key.clone()),
        );
        let public_api_figment = figment
            .clone()
            .merge(("operator_public_keys", operator_public_keys.clone()))
            .merge(("api_enabled", public_api_enabled))
            .merge(("domain", public_api_domain))
            .merge(("port", public_api_port))
            .merge(("static_path", public_api_static_path))
            .merge(("template_dir", public_api_template_path))
            .merge(("secret_key", public_api_secret_key))
            .merge(("network", args.network.clone()));

        let operator_api_enabled = if args.operator_api_enabled {
            true
        } else {
            figment
                .extract_inner("operator_api_enabled")
                .unwrap_or(true)
        };
        let operator_api_domain = args.operator_api_domain.clone().unwrap_or(
            figment
                .extract_inner("operator_api_domain")
                .unwrap_or("http://127.0.0.1:9801".to_owned()),
        );
        let operator_api_port = args
            .operator_api_port
            .unwrap_or(figment.extract_inner("operator_api_port").unwrap_or(9801));
        let operator_api_static_path = args.operator_api_static_path.clone().unwrap_or(
            figment
                .extract_inner("operator_api_static_path")
                .unwrap_or(PathBuf::from(rocket::fs::relative!(
                    "../hexstody-operator/static/"
                ))),
        );
        let operator_api_template_path = args.operator_api_template_path.clone().unwrap_or(
            figment
                .extract_inner("operator_api_template_path")
                .unwrap_or(PathBuf::from(rocket::fs::relative!(
                    "../hexstody-operator/templates/"
                ))),
        );
        let operator_api_secret_key = args.operator_api_secret_key.clone().unwrap_or(
            figment
                .extract_inner("operator_api_secret_key")
                .unwrap_or(default_secret_key.clone()),
        );
        let operator_api_figment = figment
            .clone()
            .merge(("operator_public_keys", operator_public_keys.clone()))
            .merge(("api_enabled", operator_api_enabled))
            .merge(("domain", operator_api_domain))
            .merge(("port", operator_api_port))
            .merge(("static_path", operator_api_static_path))
            .merge(("template_dir", operator_api_template_path))
            .merge(("secret_key", operator_api_secret_key))
            .merge(("network", args.network.clone()));

        ApiConfig {
            public_api_config: public_api_figment,
            operator_api_config: operator_api_figment,
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
    runtime_state_mx: Arc<Mutex<RuntimeState>>,
    state_notify: Arc<Notify>,
    start_notify: Arc<Notify>,
    update_sender: mpsc::Sender<StateUpdate>,
    btc_client: BtcClient,
    eth_client: EthClient,
    ticker_client: TickerClient,
    api_type: ApiType,
    api_config: Figment,
    abort_reg: AbortRegistration,
    is_test: bool,
) -> () {
    let api_enabled: bool = api_config.extract_inner("api_enabled").unwrap();
    if !api_enabled {
        info!("{api_type} API disabled");
        return ();
    };
    info!("Serving {api_type} API");
    match api_type {
        ApiType::Public => {
            serve_abortable(api_type, abort_reg, || {
                hexstody_public::api::serve_api(
                    pool.clone(),
                    state_mx.clone(),
                    runtime_state_mx.clone(),
                    state_notify.clone(),
                    start_notify.clone(),
                    update_sender.clone(),
                    btc_client,
                    eth_client,
                    ticker_client,
                    api_config,
                    is_test,
                )
            })
            .await;
        }
        ApiType::Operator => {
            serve_abortable(api_type, abort_reg, || {
                hexstody_operator::api::serve_api(
                    pool.clone(),
                    state_mx.clone(),
                    runtime_state_mx.clone(),
                    state_notify.clone(),
                    start_notify.clone(),
                    update_sender.clone(),
                    btc_client,
                    eth_client,
                    ticker_client,
                    api_config,
                )
            })
            .await;
        }
    };
}

pub async fn serve_apis(
    pool: Pool,
    state_mx: Arc<Mutex<State>>,
    runtime_state_mx: Arc<Mutex<RuntimeState>>,
    state_notify: Arc<Notify>,
    start_notify: Arc<Notify>,
    api_config: ApiConfig,
    api_abort: AbortRegistration,
    update_sender: mpsc::Sender<StateUpdate>,
    btc_client: BtcClient,
    eth_client: EthClient,
    ticker_client: TickerClient,
    is_test: bool,
) -> Result<(), Aborted> {
    let public_start = Arc::new(Notify::new());
    let operator_start = Arc::new(Notify::new());

    let (public_handle, public_abort_reg) = AbortHandle::new_pair();
    let public_api_fut = serve_api(
        pool.clone(),
        state_mx.clone(),
        runtime_state_mx.clone(),
        state_notify.clone(),
        public_start.clone(),
        update_sender.clone(),
        btc_client.clone(),
        eth_client.clone(),
        ticker_client.clone(),
        ApiType::Public,
        api_config.public_api_config,
        public_abort_reg,
        is_test,
    );
    let (operator_handle, operator_abort_reg) = AbortHandle::new_pair();
    let operator_api_fut = serve_api(
        pool,
        state_mx,
        runtime_state_mx.clone(),
        state_notify,
        operator_start.clone(),
        update_sender.clone(),
        btc_client.clone(),
        eth_client.clone(),
        ticker_client.clone(),
        ApiType::Operator,
        api_config.operator_api_config,
        operator_abort_reg,
        is_test,
    );
    let body_fut = async move {
        public_start.notified().await;
        operator_start.notified().await;
        start_notify.notify_one();
    };
    let abortable_apis =
        Abortable::new(join3(public_api_fut, operator_api_fut, body_fut), api_abort);
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
    #[error("API was aborted from outside")]
    Aborted,
}

pub async fn run_hot_wallet(
    args: &Args,
    start_notify: Arc<Notify>,
    btc_client: BtcClient,
    eth_client: EthClient,
    ticker_client: TickerClient,
    api_abort_reg: AbortRegistration,
    is_test: bool,
) -> Result<(), Error> {
    info!("Connecting to database");
    let pool = create_db_pool(&args.dbconnect).await?;
    info!("Reconstructing state from database");
    let state = query_state(args.network, &pool).await?;
    let state_mx = Arc::new(Mutex::new(state));
    let runtime_state_mx = Arc::new(Mutex::new(RuntimeState::new()));
    let state_notify = Arc::new(Notify::new());
    let (update_sender, update_receiver) = mpsc::channel(1000);
    let (update_resp_sender, update_resp_receiver) = mpsc::channel(1000);
    let api_config = ApiConfig::parse_figment(args);

    let update_worker_hndl = tokio::spawn({
        let pool = pool.clone();
        let state_mx = state_mx.clone();
        let state_notify = state_notify.clone();
        async move {
            update_worker(
                pool,
                state_mx,
                state_notify,
                update_receiver,
                update_resp_sender,
            )
            .await;
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

    let update_response_hndl = tokio::spawn({
        let state_mx = state_mx.clone();
        let btc_client = btc_client.clone();
        let eth_client = eth_client.clone();
        let update_sender = update_sender.clone();
        async move {
            update_results_worker(
                btc_client,
                eth_client,
                state_mx,
                update_resp_receiver,
                update_sender,
            )
            .await;
        }
    });

    let cron_workers_hndl = tokio::spawn({
        let update_sender = update_sender.clone();
        async move { cron_workers(update_sender).await }
    });

    let ticker_worker_hndl = tokio::spawn({
        let ticker_client = ticker_client.clone();
        let runtime_state_mx = runtime_state_mx.clone();
        async move { ticker_worker(ticker_client, runtime_state_mx).await }
    });

    if let Err(Aborted) = serve_apis(
        pool,
        state_mx,
        runtime_state_mx,
        state_notify,
        start_notify,
        api_config,
        api_abort_reg,
        update_sender,
        btc_client,
        eth_client,
        ticker_client,
        is_test,
    )
    .await
    {
        info!("Logic aborted, exiting...");
        update_worker_hndl.abort();
        btc_worker_hndl.abort();
        update_response_hndl.abort();
        cron_workers_hndl.abort();
        ticker_worker_hndl.abort();
        Err(Error::Aborted)
    } else {
        Ok(())
    }
}

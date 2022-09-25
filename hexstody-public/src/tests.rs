#[cfg(test)]
use hexstody_btc_client::client::BtcClient;
use hexstody_db::state::*;
use hexstody_db::Pool;
use hexstody_eth_client::client::EthClient;
use hexstody_ticker_provider::client::TickerClient;
use p256::PublicKey;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

use crate::api::serve_api;
use futures::Future;
use futures::FutureExt;
use futures_util::future::TryFutureExt;
use hexstody_client::client::HexstodyClient;
use rocket::fs::relative;
use std::panic::AssertUnwindSafe;
use hexstody_runtime_db::RuntimeState;

const SERVICE_TEST_PORT: u16 = 8000;
const SERVICE_TEST_HOST: &str = "127.0.0.1";

async fn run_api_test<F, Fut>(pool: Pool, test_body: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let _ = env_logger::builder().is_test(true).try_init();

    let state_mx = Arc::new(Mutex::new(State::default()));
    let state_notify = Arc::new(Notify::new());
    let start_notify = Arc::new(Notify::new());

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let (update_sender, _) = tokio::sync::mpsc::channel(1000);
    let pub_keys : Vec<PublicKey> = vec![];
    let btc_client = BtcClient::new("127.0.0.1");
    let eth_client = EthClient::new("http://127.0.0.1");
    let ticker_client = TickerClient::new("https://min-api.cryptocompare.com");
    let runtime_state = Arc::new(Mutex::new(RuntimeState::new()));
    let api_config = rocket::Config::figment()
        .merge(("port", SERVICE_TEST_PORT))
        .merge(("static_path", relative!("static")))
        .merge(("template_dir", "templates/"))
        .merge(("secret_key", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".to_owned()))
        .merge(("domain", format!("http://localhost:{SERVICE_TEST_PORT}")))
        .merge(("operator_public_keys", pub_keys));
    tokio::spawn({
        let state = state_mx.clone();
        let state_notify = state_notify.clone();
        let start_notify = start_notify.clone();
        async move {
            let serve_task = serve_api(
                pool,
                state,
                runtime_state,
                state_notify,
                start_notify,
                update_sender,
                btc_client,
                eth_client,
                ticker_client,
                api_config,
                true
            );
            futures::pin_mut!(serve_task);
            futures::future::select(serve_task, receiver.map_err(drop)).await;
        }
    });
    start_notify.notified().await;

    let res = AssertUnwindSafe(test_body()).catch_unwind().await;

    sender.send(()).unwrap();

    assert!(res.is_ok());
}

#[sqlx_database_tester::test(pool(variable = "pool", migrations = "../hexstody-db/migrations"))]
async fn test_public_api_ping() {
    run_api_test(pool, || async {
        let client = HexstodyClient::new(&format!(
            "http://{}:{}",
            SERVICE_TEST_HOST, SERVICE_TEST_PORT
        ), &format!("http://{}:{}", SERVICE_TEST_HOST, SERVICE_TEST_PORT + 1))
        .expect("client created");
        client.ping().await.unwrap();
    })
    .await;
}

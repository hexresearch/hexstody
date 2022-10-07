use bitcoin::network::constants::Network;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use futures::FutureExt;
use log::*;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use p256::{pkcs8::DecodePublicKey, PublicKey};
use port_selector::random_free_tcp_port;
use std::fs;
use std::future::Future;
use std::net::{IpAddr, Ipv4Addr};
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tempdir::TempDir;
use tokio::sync::{Mutex, Notify};

use hexstody_btc::api::public::serve_public_api;
use hexstody_btc::constants::CONFIRMATIONS_CONFIG;
use hexstody_btc::state::ScanState;
use hexstody_btc::worker::{cold_wallet_worker, node_worker};
use hexstody_btc_client::client::BtcClient;

fn setup_node(port: u16, rpc_port: u16) -> (Child, TempDir) {
    info!(
        "Starting regtest node on ports: {}, {} (RPC)",
        port, rpc_port
    );
    let tmp_dir = TempDir::new("regtest-data").expect("temporary data dir created");
    let node_handle = Command::new("bitcoind")
        .arg("-regtest")
        .arg("-server")
        .arg("-rpcuser=regtest")
        .arg("-rpcpassword=regtest")
        .arg("-fallbackfee=0.000002")
        .arg(format!("-port={}", port))
        .arg(format!("-rpcport={}", rpc_port))
        .arg(format!("-datadir={}", tmp_dir.path().to_str().unwrap()))
        .stdout(Stdio::null())
        .spawn()
        .expect("bitcoin node starts");
    (node_handle, tmp_dir)
}

fn teardown_node(mut node_handle: Child) {
    info!("Teardown regtest node");
    signal::kill(Pid::from_raw(node_handle.id() as i32), Signal::SIGTERM).unwrap();
    node_handle.wait().expect("Node terminated");
}

async fn setup_node_ready(port: u16, rpc_port: u16) -> (Child, Client, TempDir) {
    let (node_handle, temp_dir) = setup_node(port, rpc_port);

    let rpc_url = format!("http://127.0.0.1:{rpc_port}/wallet/default");
    let client = Client::new(
        &rpc_url,
        Auth::UserPass("regtest".to_owned(), "regtest".to_owned()),
    )
    .expect("Node client");
    wait_for_node(&client).await;
    client
        .create_wallet("default", None, None, None, None)
        .map(|_| ())
        .unwrap_or_else(|e| warn!("Cannot create default wallet: {}", e));
    (node_handle, client, temp_dir)
}

async fn wait_for_node(client: &Client) -> () {
    for _ in 0..100 {
        let res = client.get_blockchain_info();
        if let Ok(_) = res {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    client
        .get_blockchain_info()
        .expect("final check on connection");
}

async fn setup_api(rpc_port: u16) -> u16 {
    let port: u16 = random_free_tcp_port().expect("available port");
    info!("Running API server on port {port}");
    let network = Network::Regtest;
    let address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let start_notify = Arc::new(Notify::new());
    let state_notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ScanState::new(Network::Regtest)));

    let make_client = || {
        let rpc_url = format!("http://127.0.0.1:{rpc_port}");
        Client::new(
            &rpc_url,
            Auth::UserPass("regtest".to_owned(), "regtest".to_owned()),
        )
        .expect("Node client")
    };
    let sk1bytes = [
        226, 143, 42, 33, 23, 231, 50, 229, 188, 25, 0, 63, 245, 176, 125, 158, 27, 252, 214, 95,
        182, 243, 70, 176, 48, 9, 105, 34, 180, 198, 131, 6,
    ];
    let sk2bytes = [
        197, 103, 161, 120, 28, 231, 101, 35, 34, 117, 53, 115, 210, 176, 147, 227, 72, 177, 3, 11,
        69, 147, 176, 246, 176, 171, 80, 1, 68, 143, 100, 96,
    ];
    let sk3bytes = [
        136, 43, 196, 241, 144, 235, 247, 160, 3, 26, 8, 234, 164, 69, 85, 59, 219, 248, 130, 95,
        240, 188, 175, 229, 43, 160, 105, 235, 187, 120, 183, 16,
    ];
    let sk1 = p256::SecretKey::from_be_bytes(&sk1bytes).unwrap();
    let sk2 = p256::SecretKey::from_be_bytes(&sk2bytes).unwrap();
    let sk3 = p256::SecretKey::from_be_bytes(&sk3bytes).unwrap();
    let pk1 = sk1.public_key();
    let pk2 = sk2.public_key();
    let pk3 = sk3.public_key();
    tokio::spawn({
        let start_notify = start_notify.clone();
        let state_notify = state_notify.clone();
        let state = state.clone();
        let polling_duration = Duration::from_secs(1);
        let client = make_client();
        async move {
            serve_public_api(
                client,
                address,
                port,
                start_notify,
                state,
                state_notify,
                polling_duration,
                None,
                vec![pk1, pk2, pk3],
                CONFIRMATIONS_CONFIG,
                "http://127.0.0.1:8080".to_owned(),
                network,
            )
            .await
            .expect("start api");
        }
    });
    tokio::spawn({
        let client = make_client();
        let polling_duration = Duration::from_millis(100);
        let tx_notify = Arc::new(Notify::new());
        async move {
            node_worker(&client, state, state_notify, polling_duration, tx_notify).await;
        }
    });

    start_notify.notified().await;
    port
}

async fn setup_cold_api(cold_amount: u64, rpc_port: u16) -> u16 {
    let port: u16 = random_free_tcp_port().expect("available port");
    info!("Running API server on port {port}");
    let network = Network::Regtest;
    let address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let start_notify = Arc::new(Notify::new());
    let state_notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ScanState::new(network)));

    let make_client = || {
        let rpc_url = format!("http://127.0.0.1:{rpc_port}");
        Client::new(
            &rpc_url,
            Auth::UserPass("regtest".to_owned(), "regtest".to_owned()),
        )
        .expect("Node client")
    };

    tokio::spawn({
        let start_notify = start_notify.clone();
        let state_notify = state_notify.clone();
        let state = state.clone();
        let polling_duration = Duration::from_secs(1);
        let client = make_client();
        async move {
            serve_public_api(
                client,
                address,
                port,
                start_notify,
                state,
                state_notify,
                polling_duration,
                None,
                vec![],
                CONFIRMATIONS_CONFIG,
                "http://127.0.0.1:8080".to_owned(),
                network,
            )
            .await
            .expect("start api");
        }
    });
    let tx_notify = Arc::new(Notify::new());
    tokio::spawn({
        let client = make_client();
        let polling_duration = Duration::from_millis(100);
        let tx_notify = tx_notify.clone();
        async move {
            node_worker(&client, state, state_notify, polling_duration, tx_notify).await;
        }
    });
    tokio::spawn({
        let client = make_client();
        let cold_amount = bitcoin::Amount::from_sat(cold_amount);
        let cold_address =
            bitcoin::Address::from_str("bcrt1qtunasj84306suy56cts988hc0rdnrmuvqgs2ee")
                .expect("Failed to parse cold address");
        async move {
            cold_wallet_worker(&client, tx_notify.clone(), cold_amount, cold_address).await;
        }
    });

    start_notify.notified().await;
    port
}

pub async fn run_cold_test<F, Fut>(cold_amount: u64, test_body: F)
where
    F: FnOnce(Client, BtcClient) -> Fut,
    Fut: Future<Output = ()>,
{
    let _ = env_logger::builder().is_test(true).try_init();
    let node_port = random_free_tcp_port().expect("available port");
    let node_rpc_port = random_free_tcp_port().expect("available port");
    let (node_handle, client, _tmp_dir) = setup_node_ready(node_port, node_rpc_port).await;
    let api_port = setup_cold_api(cold_amount, node_rpc_port).await;
    info!("Running API server on {api_port}");
    let api_client = BtcClient::new(&format!("http://127.0.0.1:{api_port}"));
    let res = AssertUnwindSafe(test_body(client, api_client))
        .catch_unwind()
        .await;
    teardown_node(node_handle);
    assert!(res.is_ok());
}

pub async fn run_test<F, Fut>(test_body: F)
where
    F: FnOnce(Client, BtcClient) -> Fut,
    Fut: Future<Output = ()>,
{
    let _ = env_logger::builder().is_test(true).try_init();
    let node_port = random_free_tcp_port().expect("available port");
    let node_rpc_port = random_free_tcp_port().expect("available port");
    let (node_handle, client, _tmp_dir) = setup_node_ready(node_port, node_rpc_port).await;
    let api_port = setup_api(node_rpc_port).await;
    info!("Running API server on {api_port}");
    let api_client = BtcClient::new(&format!("http://127.0.0.1:{api_port}"));
    let res = AssertUnwindSafe(test_body(client, api_client))
        .catch_unwind()
        .await;
    teardown_node(node_handle);
    assert!(res.is_ok());
}

pub async fn run_two_nodes_test<F, Fut>(test_body: F)
where
    F: FnOnce(Client, Client, BtcClient) -> Fut,
    Fut: Future<Output = ()>,
{
    let _ = env_logger::builder().is_test(true).try_init();

    let node_1_port = random_free_tcp_port().expect("available port");
    let node_1_rpc_port = random_free_tcp_port().expect("available port");
    let (node_1_handle, client_1, _tmp1) = setup_node_ready(node_1_port, node_1_rpc_port).await;

    let node_2_port = random_free_tcp_port().expect("available port");
    let node_2_rpc_port = random_free_tcp_port().expect("available port");
    let (node_2_handle, client_2, _tmp2) = setup_node_ready(node_2_port, node_2_rpc_port).await;

    client_1
        .add_node(&format!("127.0.0.1:{node_2_port}"))
        .unwrap();

    let api_port = setup_api(node_1_rpc_port).await;
    info!("Running API server on {api_port}");

    let api_client = BtcClient::new(&format!("http://127.0.0.1:{api_port}"));

    let res = AssertUnwindSafe(test_body(client_1, client_2, api_client))
        .catch_unwind()
        .await;
    teardown_node(node_1_handle);
    teardown_node(node_2_handle);
    assert!(res.is_ok());
}

/// This function is similar to `setup_api`
/// but provides some configuration
async fn setup_api_regtest(
    operator_api_domain: String,
    operator_public_keys: Vec<PublicKey>,
    rpc_port: u16,
    api_port: u16,
    polling_duration: Duration,
) -> () {
    info!("Running API server on port {api_port}");
    let network = Network::Regtest;
    let address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let start_notify = Arc::new(Notify::new());
    let state_notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ScanState::new(network)));

    let make_client = || {
        let rpc_url = format!("http://127.0.0.1:{rpc_port}/wallet/default");
        Client::new(
            &rpc_url,
            Auth::UserPass("regtest".to_owned(), "regtest".to_owned()),
        )
        .expect("Node client")
    };

    tokio::spawn({
        let start_notify = start_notify.clone();
        let state_notify = state_notify.clone();
        let state = state.clone();
        let client = make_client();
        async move {
            serve_public_api(
                client,
                address,
                api_port,
                start_notify,
                state,
                state_notify,
                polling_duration,
                None,
                operator_public_keys,
                CONFIRMATIONS_CONFIG,
                operator_api_domain,
                network,
            )
            .await
            .expect("start api");
        }
    });
    tokio::spawn({
        let client = make_client();
        let polling_duration = Duration::from_secs(30);
        let tx_notify = Arc::new(Notify::new());
        async move {
            node_worker(&client, state, state_notify, polling_duration, tx_notify).await;
        }
    });

    start_notify.notified().await;
}

// This function is used for manual testing.
// It starts 2 BTC regtest nodes so we can top up the
// user's balance from an external wallet.
// It also starts instance of hexstody-btc API.
pub async fn run_regtest<F, Fut>(
    operator_api_domain: Option<String>,
    operator_public_key_paths: Vec<PathBuf>,
    body: F,
) where
    F: FnOnce((u16, Client), (u16, Client), (String, BtcClient)) -> Fut,
    Fut: Future<Output = ()>,
{
    // Start 1st BTC node
    let node_1_port = 9803;
    let node_1_rpc_port = 9804;
    let (node_1_handle, client_1, _tmp1) = setup_node_ready(node_1_port, node_1_rpc_port).await;

    // Start 2nd BTC node
    let node_2_port = 9805;
    let node_2_rpc_port = 9806;
    let (node_2_handle, client_2, _tmp2) = setup_node_ready(node_2_port, node_2_rpc_port).await;

    // Connect them together
    client_1
        .add_node(&format!("127.0.0.1:{node_2_port}"))
        .unwrap_or_else(|e| info!("Failed to connect nodes: {}!", e));

    // Parse operator API args
    let operator_api_domain = operator_api_domain.unwrap_or("http://127.0.0.1:9801".to_owned());
    let mut operator_public_keys = vec![];
    for p in operator_public_key_paths {
        let full_path = fs::canonicalize(&p).expect("Something went wrong reading the file");
        let key_str = fs::read_to_string(full_path).expect("Something went wrong reading the file");
        let public_key = PublicKey::from_public_key_pem(&key_str)
            .expect("Something went wrong decoding the key file");
        operator_public_keys.push(public_key);
    }
    // Start hexstody-btc API and connect it to 1st BTC node.
    let api_port = 9802;
    let polling_duration = Duration::from_secs(300);
    setup_api_regtest(
        operator_api_domain,
        operator_public_keys,
        node_1_rpc_port,
        api_port,
        polling_duration,
    )
    .await;
    let api_url = format!("http://127.0.0.1:{api_port}");
    let api_client = BtcClient::new(&api_url);

    body(
        (node_1_rpc_port, client_1),
        (node_2_rpc_port, client_2),
        (api_url, api_client),
    )
    .await;
    teardown_node(node_1_handle);
    teardown_node(node_2_handle);
}

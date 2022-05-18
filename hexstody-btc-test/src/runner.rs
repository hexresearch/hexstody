use bitcoin::network::constants::Network;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use futures::FutureExt;
use hexstody_btc::api::public::serve_public_api;
use hexstody_btc::state::ScanState;
use hexstody_btc::worker::node_worker;
use hexstody_btc_client::client::BtcClient;
use log::*;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use port_selector::random_free_tcp_port;
use std::future::Future;
use std::net::{IpAddr, Ipv4Addr};
use std::panic::AssertUnwindSafe;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tempdir::TempDir;
use tokio::sync::{Mutex, Notify};

fn setup_node() -> (Child, u16, TempDir) {
    info!("Starting regtest node");
    let tmp_dir = TempDir::new("regtest-data").expect("temporary data dir crated");
    let rpc_port: u16 = random_free_tcp_port().expect("available port");

    let node_handle = Command::new("bitcoind")
        .arg("-regtest")
        .arg("-server")
        .arg("-listen=0")
        .arg("-rpcuser=regtest")
        .arg("-rpcpassword=regtest")
        .arg("-fallbackfee=0.000002")
        .arg(format!("-rpcport={}", rpc_port))
        .arg(format!("-datadir={}", tmp_dir.path().to_str().unwrap()))
        .stdout(Stdio::null())
        .spawn()
        .expect("bitcoin node starts");

    (node_handle, rpc_port, tmp_dir)
}

fn teardown_node(mut node_handle: Child) {
    info!("Teardown regtest node");
    signal::kill(Pid::from_raw(node_handle.id() as i32), Signal::SIGTERM).unwrap();
    node_handle.wait().expect("Node terminated");
}

async fn setup_node_ready() -> (Child, Client, u16, TempDir) {
    let _ = env_logger::builder().is_test(true).try_init();
    let (node_handle, rpc_port, temp_dir) = setup_node();
    info!("Running first bitcoin node on {rpc_port}");
    let rpc_url = format!("http://127.0.0.1:{rpc_port}");
    let client = Client::new(
        &rpc_url,
        Auth::UserPass("regtest".to_owned(), "regtest".to_owned()),
    )
    .expect("Node client");
    wait_for_node(&client).await;
    client
        .create_wallet("", None, None, None, None)
        .expect("create default wallet");
    (node_handle, client, rpc_port, temp_dir)
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

    tokio::spawn({
        let start_notify = start_notify.clone();
        let state_notify = state_notify.clone();
        let state = state.clone();
        let polling_duration = Duration::from_secs(1);
        let client = make_client();
        // 64 0es encoded to base64
        let secret_key = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
        async move {
            serve_public_api(
                client,
                address,
                port,
                start_notify,
                state,
                state_notify,
                polling_duration,
                secret_key,
            )
            .await
            .expect("start api");
        }
    });
    tokio::spawn({
        let client = make_client();
        let polling_duration = Duration::from_millis(100);
        async move {
            node_worker(&client, state, state_notify, polling_duration).await;
        }
    });

    start_notify.notified().await;
    port
}

pub async fn run_test<F, Fut>(test_body: F)
where
    F: FnOnce(Client, BtcClient) -> Fut,
    Fut: Future<Output = ()>,
{
    let _ = env_logger::builder().is_test(true).try_init();
    let (node_handle, client, rpc_port, _tmp_dir) = setup_node_ready().await;

    let api_port = setup_api(rpc_port).await;
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
    let (node1_handle, client1, rpc_port, _tmp1) = setup_node_ready().await;
    let (node2_handle, client2, _, _tmp2) = setup_node_ready().await;

    let api_port = setup_api(rpc_port).await;
    info!("Running API server on {api_port}");

    let api_client = BtcClient::new(&format!("http://127.0.0.1:{api_port}"));

    let res = AssertUnwindSafe(test_body(client1, client2, api_client))
        .catch_unwind()
        .await;
    teardown_node(node1_handle);
    teardown_node(node2_handle);
    assert!(res.is_ok());
}

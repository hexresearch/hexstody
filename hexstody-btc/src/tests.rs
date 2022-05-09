use futures::FutureExt;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::process::{Child, Command};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use tempdir::TempDir;
use log::*;
use rand::{thread_rng, Rng};

fn setup() -> (Child, u16, TempDir) {
    println!("Starting regtest node");
    let tmp_dir = TempDir::new("regtest-data").expect("temporary data dir crated");
    let mut rng = thread_rng();
    let rpc_port: u16 = rng.gen_range(10000 .. u16::MAX);

    let node_handle = Command::new("bitcoind")
        .arg("-regtest")
        .arg("-server")
        .arg("-rpcuser=regtest")
        .arg("-rpcpassword=regtest")
        .arg(format!("-rpcport={}", rpc_port))
        .arg(format!("-datadir={}", tmp_dir.path().to_str().unwrap()))
        .spawn()
        .expect("bitcoin node starts");

    (node_handle, rpc_port, tmp_dir)
}

fn teardown(mut node_handle: Child) {
    println!("Teardown regtest node");
    signal::kill(Pid::from_raw(node_handle.id() as i32), Signal::SIGTERM).unwrap();
    node_handle.wait().expect("Node terminated");
}

async fn run_test<F, Fut>(test_body: F)
where
    F: FnOnce(Client) -> Fut,
    Fut: Future<Output = ()>,
{
    let _ = env_logger::builder().is_test(true).try_init();
    let (node_handle, rpc_port, _temp_dir) = setup();
    info!("Running bitcoin node on {rpc_port}");
    let rpc_url = format!("http://127.0.0.1:{rpc_port}");
    let client = Client::new(&rpc_url, Auth::UserPass("regtest".to_owned(), "regtest".to_owned())).expect("Node client");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let res = AssertUnwindSafe(test_body(client)).catch_unwind().await;
    teardown(node_handle);
    assert!(res.is_ok());
}

#[tokio::test]
async fn basic_test() {
    run_test(|client| async move { 
        println!("Running basic test");
        let info = client.get_blockchain_info().expect("blockchain info");
        assert_eq!(info.chain, "regtest");
    }).await;
}

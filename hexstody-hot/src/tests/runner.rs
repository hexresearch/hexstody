use crate::runner::{run_hot_wallet, ApiConfig};
use bitcoincore_rpc::Client;
use futures::FutureExt;
use hexstody_btc_client::client::BtcClient;
use hexstody_btc_test::runner as btc_runner;
use hexstody_client::client::HexstodyClient;
use log::*;
use port_selector::random_free_tcp_port;
use run_script::ScriptOptions;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::Duration;
use tempdir::TempDir;
use tokio::sync::Notify;

pub struct TestEnv {
    pub btc_node: Client,
    pub other_btc_node: Client,
    pub btc_adapter: BtcClient,
    pub hot_client: HexstodyClient,
}

pub async fn run_test<F, Fut>(test_body: F)
where
    F: FnOnce(TestEnv) -> Fut,
    Fut: Future<Output = ()>,
{
    btc_runner::run_two_nodes_test(|btc_node, other_btc_node, btc_adapter| async move {
        let (db_port, db_dir) = setup_postgres();
        let dbconnect = format!("postgres://hexstody:hexstody@localhost:{db_port}/hexstody");
        info!("Connection to database: {dbconnect}");
        let public_api_port: u16 = random_free_tcp_port().expect("available port");
        let operator_api_port: u16 = random_free_tcp_port().expect("available port");

        let start_notify = Arc::new(Notify::new());
        let api_handle = tokio::spawn({
            let start_notify = start_notify.clone();
            async move {
                let mut api_config = ApiConfig::parse_figment();
                api_config.public_api_port = public_api_port;
                api_config.operator_api_port = operator_api_port;
                match run_hot_wallet(api_config, &dbconnect, start_notify).await {
                    Err(e) => {
                        error!("Hot wallet error: {e}");
                    }
                    _ => {
                        info!("Terminated gracefully!");
                    }
                }
            }
        });

        let hot_client = HexstodyClient::new(&format!("http://localhost:{public_api_port}"));
        let env = TestEnv {
            btc_node,
            other_btc_node,
            btc_adapter,
            hot_client,
        };
        tokio::time::timeout(Duration::from_secs(2), start_notify.notified())
            .await
            .expect("timeout");

        let res = AssertUnwindSafe(test_body(env)).catch_unwind().await;

        api_handle.abort();
        teardown_postgres(&db_dir, db_port);
        assert!(res.is_ok());
    })
    .await;
}

fn setup_postgres() -> (u16, TempDir) {
    info!("Starting PostgreSQL");
    let tmp_dir = TempDir::new("pg-data").expect("temporary data dir crated");
    let db_port: u16 = random_free_tcp_port().expect("available port");

    let options = ScriptOptions::new();
    // options.output_redirection = run_script::types::IoOptions::Inherit;
    let (code, output, error) = run_script::run_script!(
        r#"
         export PGDATA=$1
         export PGPORT=$2
         initdb $PGDATA --auth=trust
         echo "unix_socket_directories = '$PGDATA'" >> $PGDATA/postgresql.conf
         pg_ctl start -D$PGDATA -l $PGDATA/psqlog
         psql --host=$PGDATA -d postgres -c "create role \"hexstody\" with login password 'hexstody';"
         psql --host=$PGDATA -d postgres -c "create database \"hexstody\" owner \"hexstody\";"
         "#,
        &vec![tmp_dir.path().to_str().unwrap().to_owned(), format!("{db_port}")],
        &options
    )
    .unwrap();

    info!("Exit Code: {}", code);
    if code != 0 {
        info!("Output: {}", output);
        info!("Error: {}", error);
    }

    (db_port, tmp_dir)
}

fn teardown_postgres(tmp_dir: &TempDir, db_port: u16) {
    println!("Teardown PostgreSQL");
    let options = ScriptOptions::new();
    // options.output_redirection = run_script::types::IoOptions::Inherit;
    let (code, output, error) = run_script::run_script!(
        r#"
         export PGDATA=$1
         export PGPORT=$2
         pg_ctl stop -D$PGDATA
        "#,
        &vec![
            tmp_dir.path().to_str().unwrap().to_owned(),
            format!("{db_port}")
        ],
        &options
    )
    .unwrap();

    info!("Exit Code: {}", code);
    if code != 0 {
        info!("Output: {}", output);
        info!("Error: {}", error);
    }
}

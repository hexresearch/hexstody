use bitcoincore_rpc::Client;
use futures::{future::AbortHandle, FutureExt};
use hexstody_eth_client::client::EthClient;
use hexstody_ticker_provider::client::TickerClient;
use log::*;
use p256::pkcs8::EncodePublicKey;
use p256::SecretKey;
use anyhow::Context;
use port_selector::random_free_tcp_port;
use run_script::ScriptOptions;
use std::future::Future;
use std::io::Write;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempdir::TempDir;
use tokio::sync::Notify;

use hexstody_api::types::{SigninEmail, SignupEmail, InviteRequest};
use hexstody_btc_client::client::BtcClient;
use hexstody_btc_test::runner as btc_runner;
use hexstody_client::client::HexstodyClient;
use hexstody_db::state::Network;
use hexstody_db::update::signup::UserId;

use crate::runner::run_hot_wallet;
use crate::{Args, SubCommand};

pub struct TestEnv {
    pub btc_node: Client,
    pub other_btc_node: Client,
    pub btc_client: BtcClient,
    pub hot_client: HexstodyClient,
    pub secret_keys: Vec<SecretKey>
}

pub async fn run_test<F, Fut>(test_body: F)
where
    F: FnOnce(TestEnv) -> Fut,
    Fut: Future<Output = ()>,
{
    btc_runner::run_two_nodes_test(|btc_node, other_btc_node, btc_client| async move {
        let eth_module = "http://127.0.0.1:8540".to_owned();
        let eth_client = EthClient::new(&eth_module);
        let (db_port, db_dir) = setup_postgres();
        let dbconnect = format!("postgres://hexstody:hexstody@localhost:{db_port}/hexstody");
        info!("Connection to database: {dbconnect}");
        let public_api_port: u16 = random_free_tcp_port().expect("available port");
        let operator_api_port: u16 = random_free_tcp_port().expect("available port");

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
        let pub_keys = vec![("/tmp/hexstody/pk1", pk1),("/tmp/hexstody/pk2", pk2),("/tmp/hexstody/pk3", pk3)];
        let secret_keys = vec![sk1,sk2,sk3];
        let _ = std::fs::create_dir_all("/tmp/hexstody");
        let mut keys = vec![];
        for (name, k) in pub_keys.iter() {
            let public_key = k.clone();
            let mut pub_key_path: PathBuf = name.to_string().into();
            pub_key_path.set_extension("pub.pem");
            let path = pub_key_path.clone();
            let mut pub_key_file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(pub_key_path)
                .with_context(|| format!("Failed to open file {}", path.display())).unwrap();
            let encoded_public_key = public_key.to_public_key_pem(Default::default()).unwrap();
            pub_key_file.write_all(encoded_public_key.as_bytes()).unwrap();
            keys.push(path.clone());
        }
        let start_notify = Arc::new(Notify::new());
        let api_handle = tokio::spawn({
            let start_notify = start_notify.clone();
            let btc_client = btc_client.clone();
            let ticker_client = TickerClient::new("https://min-api.cryptocompare.com");
            async move {
                let (_, abort_reg) = AbortHandle::new_pair();
                match run_hot_wallet(
                    &Args {
                        dbconnect: dbconnect,
                        btc_module: "http://127.0.0.1:8180".to_owned(),
                        eth_module: eth_module,
                        ticker_provider: "https://min-api.cryptocompare.com".to_owned(),
                        network: Network::Regtest,
                        start_regtest: true,
                        operator_public_keys: keys,
                        public_api_enabled: true,
                        public_api_domain: Some(format!("http://localhost:{public_api_port}")),
                        public_api_port: Some(public_api_port),
                        public_api_static_path: None,
                        public_api_template_path: None,
                        public_api_secret_key: None,
                        operator_api_enabled: true,
                        operator_api_domain: Some(format!("http://localhost:{operator_api_port}")),
                        operator_api_port: Some(operator_api_port),
                        operator_api_static_path: None,
                        operator_api_template_path: None,
                        operator_api_secret_key: None,
                        subcmd: SubCommand::Serve,
                    },
                    start_notify,
                    btc_client,
                    eth_client,
                    ticker_client,
                    abort_reg,
                    true
                )
                .await
                {
                    Err(e) => {
                        error!("API error: {e}");
                    }
                    _ => {
                        info!("Terminated gracefully!");
                    }
                }
            }
        });

        let hot_client = HexstodyClient::new(&format!("http://localhost:{public_api_port}"), &format!("http://localhost:{operator_api_port}"))
            .expect("cleint created");
        let env = TestEnv {
            btc_node,
            other_btc_node,
            btc_client,
            hot_client,
            secret_keys
        };
        tokio::time::timeout(Duration::from_secs(2), start_notify.notified())
            .await
            .expect("timeout");

        let res = AssertUnwindSafe(test_body(env)).catch_unwind().await;

        api_handle.abort();
        teardown_postgres(&db_dir, db_port);

        for (name,_) in pub_keys.iter() {
            let mut pub_key_path: PathBuf = name.to_string().into();
            pub_key_path.set_extension("pub.pem");
            std::fs::remove_file(pub_key_path).expect("failed to delete keyfile");
        }
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
         pg_ctl start -D$PGDATA -l $PGDATA/postgresql.log
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

pub struct LoggedTestEnv {
    pub btc_node: Client,
    pub other_btc_node: Client,
    pub btc_client: BtcClient,
    pub hot_client: HexstodyClient,
    pub user_id: UserId,
}

pub async fn run_with_user<F, Fut>(test_body: F)
where
    F: FnOnce(LoggedTestEnv) -> Fut,
    Fut: Future<Output = ()>,
{
    run_test(|env| async move {
        let user = "aboba@mail.com".to_owned();
        let password = "123456".to_owned();
        let _removed = env.hot_client.test_only_remove_eth_user(&user).await;
        let sk = env.secret_keys.get(0).unwrap();
        let invite_req = InviteRequest{ label: "test invite".to_string() };
        let invite_resp = env.hot_client.gen_invite(sk.clone(), invite_req).await.expect("Failed to register invite");
        env.hot_client
            .signup_email(SignupEmail {
                user: user.clone(),
                invite: invite_resp.invite,
                password: password.clone(),
            })
            .await
            .expect("Signup");

        env.hot_client
            .signin_email(SigninEmail {
                user: user.clone(),
                password: password.clone(),
            })
            .await
            .expect("Signin");

        let logged_env = LoggedTestEnv {
            btc_node: env.btc_node,
            other_btc_node: env.other_btc_node,
            btc_client: env.btc_client,
            hot_client: env.hot_client.clone(),
            user_id: user.clone(),
        };
        test_body(logged_env).await;

        env.hot_client.logout().await.expect("Logout");
    })
    .await;
}

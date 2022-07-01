mod api;
mod state;
mod worker;

use bitcoin::network::constants::Network;
use bitcoincore_rpc::{Auth, Client};
use clap::Parser;
use futures::future::try_join;
use futures::future::{AbortHandle, Abortable, Aborted};
use log::*;
use std::error::Error;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use std::path::PathBuf;
use std::fs;
use thiserror::Error;
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;
use p256::PublicKey;
use p256::pkcs8::DecodePublicKey;

use api::public::*;
use state::ScanState;
use worker::node_worker;

// Should be the same as hexstody-db::state::REQUIRED_NUMBER_OF_CONFIRMATIONS
pub const REQUIRED_NUMBER_OF_CONFIRMATIONS: i16 = 2;

#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    /// Start listening incoming API requests
    Serve {
        #[clap(long, short, default_value = "8180", env = "HEXSTODY_BTC_API_PORT")]
        port: u16,
        #[clap(
            long,
            short,
            default_value = "127.0.0.1",
            env = "HEXSTODY_BTC_API_ADDRESS"
        )]
        address: IpAddr,
        #[clap(
            long,
            default_value = "http://127.0.0.1:8332",
            env = "HEXSTODY_BTC_NODE_URL"
        )]
        node_url: String,
        #[clap(long, default_value = "user", env = "HEXSTODY_BTC_NODE_USER")]
        node_user: String,
        #[clap(long, env = "HEXSTODY_BTC_NODE_PASSWORD", hide_env_values = true)]
        node_password: String,
        #[clap(long, default_value = "bitcoin", env = "HEXSTODY_BTC_NODE_NETWORK")]
        network: Network,
        /// Base64 encoded 64 bytes for encoding cookies. Required in release profile.
        #[clap(long, env = "HEXSTODY_BTC_SECRET_KEY", hide_env_values = true)]
        secret_key: Option<String>,
        #[clap(
            long,
            env = "HEXSTODY_OPERATOR_PUBLIC_KEYS",
            takes_value = true,
            multiple_values = true,
            min_values = usize::try_from(REQUIRED_NUMBER_OF_CONFIRMATIONS).unwrap(),
            required = true
        )]
        /// List of paths to files containing trusted public keys, which operators use to confirm withdrawal requests
        operator_public_keys: Vec<PathBuf>,
        #[clap(long, env="HEXSTODY_BTC_HOT_DOMAIN")]
        hot_domain: String,
    },
}

#[derive(Debug, Error)]
enum LogicError {
    #[error("API error: {0}")]
    Api(#[from] rocket::Error),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    match args.subcmd.clone() {
        SubCommand::Serve {
            address,
            port,
            node_url,
            node_user,
            node_password,
            network,
            secret_key,
            operator_public_keys,
            hot_domain,
        } => loop {
            let mut op_public_keys = vec![]; //args.op_public_keys.clone()
            for p in &operator_public_keys {
                let full_path = fs::canonicalize(&p).expect("Something went wrong reading the file");
                let key_str =
                    fs::read_to_string(full_path).expect("Something went wrong reading the file");
                let public_key = PublicKey::from_public_key_pem(&key_str)
                    .expect("Something went wrong decoding the key file");
                op_public_keys.push(public_key);
            }
            let (abort_handle, abort_reg) = AbortHandle::new_pair();
            ctrlc::set_handler(move || {
                abort_handle.abort();
            })
            .expect("Error setting Ctrl-C handler");

            let make_client = || {
                Client::new(
                    &node_url,
                    Auth::UserPass(node_user.clone(), node_password.clone()),
                )
                .expect("Node client")
            };
            let state = Arc::new(Mutex::new(ScanState::new(network)));
            let state_notify = Arc::new(Notify::new());
            let polling_duration = Duration::from_secs(30);
            let worker_fut = async {
                let client = make_client();
                let res = node_worker(
                    &client,
                    state.clone(),
                    state_notify.clone(),
                    polling_duration,
                )
                .await;
                Ok(res)
            };

            let start_notify = Arc::new(Notify::new());
            let public_api_fut = async {
                let client = make_client();
                let res = serve_public_api(
                    client,
                    address,
                    port,
                    start_notify,
                    state.clone(),
                    state_notify.clone(),
                    polling_duration,
                    secret_key.as_deref(),
                    op_public_keys,
                    REQUIRED_NUMBER_OF_CONFIRMATIONS,
                    hot_domain.clone()
                )
                .await;
                res.map_err(|err| LogicError::from(err))
            };

            let joined_fut = try_join(worker_fut, public_api_fut);
            match Abortable::new(joined_fut, abort_reg).await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    error!("Terminated with: {e}");
                }
                Err(Aborted) => {
                    error!("API and workers aborted");
                    return Ok(());
                }
            }
            let restart_dt = Duration::from_secs(5);
            info!("Adding {:?} delay before restarting logic", restart_dt);
            sleep(restart_dt).await;
        },
    }
}

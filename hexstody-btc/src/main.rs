mod api;
mod state;
#[cfg(test)]
mod tests;
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
use thiserror::Error;
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;

use api::public::*;
use state::ScanState;
use worker::node_worker;

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
        } => loop {
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

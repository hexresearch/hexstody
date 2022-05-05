mod api;
mod worker;

use clap::Parser;
use futures::future::{AbortHandle, Abortable, Aborted};
use log::*;
use std::error::Error;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::sleep;
use futures::future::try_join;
use thiserror::Error;

use api::public::*;
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
    },
}

#[derive(Debug, Error)]
enum LogicError {
    #[error("Node worker error: {0}")]
    Worker(#[from] worker::Error),
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
        SubCommand::Serve { address, port } => loop {
            let (abort_handle, abort_reg) = AbortHandle::new_pair();
            ctrlc::set_handler(move || {
                abort_handle.abort();
            })
            .expect("Error setting Ctrl-C handler");

            let worker_fut = async {
                let res = node_worker().await;
                res.map_err(|err| LogicError::from(err))
            };
            
            let start_notify = Arc::new(Notify::new());
            let public_api_fut = async {
                let res = serve_public_api(address, port, start_notify).await;
                res.map_err(|err| LogicError::from(err))
            };

            let joined_fut = try_join(worker_fut, public_api_fut);
            match Abortable::new(joined_fut, abort_reg).await {
                Ok(Ok(_)) => {},
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

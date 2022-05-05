mod api;

use clap::Parser;
use futures::future::{AbortHandle, Abortable, Aborted};
use log::*;
use std::error::Error;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::sleep;

use api::public::*;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    match args.subcmd.clone() {
        SubCommand::Serve { address, port } => loop {
            let (abort_api_handle, abort_api_reg) = AbortHandle::new_pair();
            ctrlc::set_handler(move || abort_api_handle.abort())
                .expect("Error setting Ctrl-C handler");

            info!("Serving API");
            let start_notify = Arc::new(Notify::new());
            let public_api_fut = tokio::spawn(serve_public_api(address, port, start_notify));
            match Abortable::new(public_api_fut, abort_api_reg).await {
                Ok(_) => (),
                Err(Aborted) => {
                    error!("API thread aborted");
                    return Ok(());
                }
            }

            let restart_dt = Duration::from_secs(5);
            info!("Adding {:?} delay before restarting logic", restart_dt);
            sleep(restart_dt).await;
        },
    }
}

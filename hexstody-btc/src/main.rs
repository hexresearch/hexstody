mod api;

use clap::Parser;
use futures::future::{AbortHandle, Abortable, Aborted};
use log::*;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
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
    Serve
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    env_logger::init();

    match args.subcmd.clone() {
        SubCommand::Serve => loop {
            let args = args.clone();
            let (_abort_api_handle, abort_api_reg) = AbortHandle::new_pair();

            info!("Serving API");

            let public_api_fut = tokio::spawn(serve_public_api());
            match Abortable::new(public_api_fut, abort_api_reg).await {
                Ok(mres) => (),
                Err(Aborted) => {
                    error!("API thread aborted")
                }
            }

            let restart_dt = Duration::from_secs(5);
            info!("Adding {:?} delay before restarting logic", restart_dt);
            sleep(restart_dt).await;
        }
    }
    Ok(())
}

mod api;
mod runner;

use clap::Parser;
use futures::future::AbortHandle;
use hexstody_db::state::Network;
use log::*;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::sleep;

use runner::run_api;

#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
struct Args {
    // #[clap(long, env = "KOLLIDER_API_KEY", hide_env_values = true)]
    // api_key: String,
    /// PostgreSQL connection string
    #[clap(
        long,
        short,
        default_value = "postgres://hexstody:hexstody@localhost/hexstody",
        env = "DATABASE_URL"
    )]
    dbconnect: String,
    #[clap(long, default_value = "mainnet", env = "HEXSTODY_NETWORK")]
    network: Network,
    #[clap(long, env = "HEXSTODY_START_REGTEST")]
    start_regtest: bool,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    /// Start listening incoming API requests
    Serve,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    match args.subcmd.clone() {
        SubCommand::Serve => run(&args).await,
    }
    Ok(())
}

async fn run(args: &Args) {
    loop {
        let start_notify = Arc::new(Notify::new());

        let (api_abort_handle, api_abort_reg) = AbortHandle::new_pair();
        ctrlc::set_handler(move || {
            api_abort_handle.abort();
        })
        .expect("Error setting Ctrl-C handler");
        match run_api(args.network, &args.dbconnect, start_notify, api_abort_reg).await {
            Err(e) => {
                error!("API error: {e}");
            }
            _ => {
                info!("Terminated gracefully!");
                return ();
            }
        }
        let restart_dt = Duration::from_secs(5);
        info!("Adding {:?} delay before restarting logic", restart_dt);
        sleep(restart_dt).await;
    }
}

mod api;
mod runner;
#[cfg(test)]
mod tests;
mod worker;

use clap::Parser;
use hexstody_btc_client::client::BtcClient;
use hexstody_db::state::Network;
use log::*;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::sleep;

use hexstody_btc_test::runner::run_test as run_btc_regtest;
use runner::{run_hot_wallet, ApiConfig};

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
    #[clap(
        long,
        short,
        default_value = "http://127.0.0.1:8180",
        env = "BTC_MODULE_URL"
    )]
    btc_module: String,
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
        SubCommand::Serve => {
            if args.start_regtest {
                run_btc_regtest(|_, btc_client| {
                    let args = args.clone();
                    async move { run(btc_client, &args).await }
                })
                .await
            } else {
                let btc_client = BtcClient::new(&args.btc_module);
                run(btc_client, &args).await
            }
        }
    }
    Ok(())
}

async fn run(btc_client: BtcClient, args: &Args) {
    loop {
        let api_config = ApiConfig::parse_figment();
        let start_notify = Arc::new(Notify::new());

        match run_hot_wallet(
            args.network,
            api_config,
            &args.dbconnect,
            start_notify,
            btc_client.clone(),
        )
        .await
        {
            Err(e) => {
                error!("Hot wallet error: {e}");
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

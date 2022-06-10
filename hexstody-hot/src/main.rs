mod runner;
#[cfg(test)]
mod tests;
mod worker;

use clap::Parser;
use futures::future::AbortHandle;
use hexstody_btc_client::client::BtcClient;
use hexstody_db::state::Network;
use log::*;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::sleep;

use hexstody_btc_test::runner::run_test as run_btc_regtest;
use runner::run_hot_wallet;

#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
pub struct Args {
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
    #[clap(
        long,
        env = "HEXSTODY_OPERATOR_PUBLIC_KEYS",
        takes_value = true,
        multiple_values = true,
        min_values = 1,
        required = true
    )]
    /// List of paths to files containing trusted public keys, which operators use to confirm withdrawal requests
    operator_public_keys: Vec<PathBuf>,
    #[clap(long, env = "HEXSTODY_PUBLIC_API_ENABLED")]
    public_api_enabled: bool,
    #[clap(long, env = "HEXSTODY_PUBLIC_API_DOMAIN")]
    public_api_domain: Option<String>,
    #[clap(long, env = "HEXSTODY_PUBLIC_API_PORT")]
    public_api_port: Option<u16>,
    #[clap(long, env = "HEXSTODY_PUBLIC_API_STATIC_PATH")]
    public_api_static_path: Option<PathBuf>,
    #[clap(long, env = "HEXSTODY_PUBLIC_API_TEMPLATE_PATH")]
    public_api_template_path: Option<PathBuf>,
    /// Base64 encoded 64 byte secret key for encoding cookies. Required in release profile.
    #[clap(long, env = "HEXSTODY_PUBLIC_API_SECRET_KEY", hide_env_values = true)]
    public_api_secret_key: Option<String>,
    #[clap(long, env = "HEXSTODY_OPERATOR_API_ENABLED")]
    operator_api_enabled: bool,
    #[clap(long, env = "HEXSTODY_OPERATOR_API_DOMAIN")]
    operator_api_domain: Option<String>,
    #[clap(long, env = "HEXSTODY_OPERATOR_API_PORT")]
    operator_api_port: Option<u16>,
    #[clap(long, env = "HEXSTODY_OPERATOR_API_STATIC_PATH")]
    operator_api_static_path: Option<PathBuf>,
    #[clap(long, env = "HEXSTODY_OPERATOR_API_TEMPLATE_PATH")]
    operator_api_template_path: Option<PathBuf>,
    /// Base64 encoded 64 byte secret key for encoding cookies. Required in release profile.
    #[clap(long, env = "HEXSTODY_OPERATOR_API_SECRET_KEY", hide_env_values = true)]
    operator_api_secret_key: Option<String>,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    /// Start listening incoming API requests
    Serve,
}

async fn run(btc_client: BtcClient, args: &Args) {
    loop {
        let start_notify = Arc::new(Notify::new());
        let (api_abort_handle, api_abort_reg) = AbortHandle::new_pair();
        // TODO: On second loop iteration `MultipleHandlers` error occures
        ctrlc::set_handler(move || {
            api_abort_handle.abort();
        })
        .expect("Error setting Ctrl-C handler: {e}");
        match run_hot_wallet(args, start_notify, btc_client.clone(), api_abort_reg).await {
            Ok(_) | Err(runner::Error::Aborted) => {
                info!("Terminated gracefully!");
                return ();
            }
            Err(e) => {
                error!("API error: {e}");
            }
        }
        let restart_dt = Duration::from_secs(5);
        info!("Adding {:?} delay before restarting logic", restart_dt);
        sleep(restart_dt).await;
    }
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
                    let mut args = args.clone();
                    args.network = Network::Regtest;
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

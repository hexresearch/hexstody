mod runner;
#[cfg(test)]
mod tests;
mod worker;

use clap::Parser;
use futures::future::{join, AbortHandle};
use hexstody_btc_client::client::BtcClient;
use hexstody_btc_test::runner::run_regtest;
use hexstody_db::state::{Network, REQUIRED_NUMBER_OF_CONFIRMATIONS};
use hexstody_eth_client::client::EthClient;
use hexstody_ticker_provider::client::TickerClient;
use log::*;
use runner::{ApiConfig, run_hot_wallet};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify;

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
    #[clap(
        long,
        short,
        default_value = "http://127.0.0.1:8540",
        env = "ETH_MODULE_URL"
    )]
    eth_module: String,
    #[clap(long, default_value = "https://min-api.cryptocompare.com", env = "HEXSTODY_TICKER_PROVIDER")]
    ticker_provider: String,
    #[clap(long, default_value = "mainnet", env = "HEXSTODY_NETWORK")]
    network: Network,
    #[clap(long, env = "HEXSTODY_START_REGTEST")]
    start_regtest: bool,
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

async fn run(
    btc_client: BtcClient,
    eth_client: EthClient,
    ticker_client: TickerClient,
    args: &Args,
    start_notify: Arc<Notify>
) {
    let (api_abort_handle, api_abort_reg) = AbortHandle::new_pair();
    ctrlc::set_handler(move || {
        api_abort_handle.abort();
    })
    .expect("Error setting Ctrl-C handler: {e}");
    match run_hot_wallet(args, start_notify, btc_client.clone(), eth_client.clone(), ticker_client.clone(), api_abort_reg, false).await {
        Ok(_) | Err(runner::Error::Aborted) => {
            info!("Terminated gracefully!");
            return ();
        }
        Err(e) => {
            error!("API error: {e}");
        }
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
            let regtest_flag = args.start_regtest;
            if false {
                run_regtest(
                    args.operator_api_domain.clone(),
                    args.operator_public_keys.clone(),
                    |(node1_port, _), (node2_port, _), (hbtc_url, btc_client)| {
                        let eth_client = EthClient::new(&args.eth_module);
                        let ticker_client = TickerClient::new(&args.ticker_provider);
                        let mut args = args.clone();
                        args.network = Network::Regtest;
                        let start_notify = Arc::new(Notify::new());
                        async move {
                            let run_fut = run(btc_client, eth_client, ticker_client, &args, start_notify);
                            let msg_fut = {
                                let args = args.clone();
                                async move {
                                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                    let api_config = ApiConfig::parse_figment(&args);
                                    let op_port: u16 = api_config.operator_api_config.extract_inner("port").expect("operators API port");
                                    let pub_port: u16 = api_config.public_api_config.extract_inner("port").expect("public API port");
                                    println!("======================== Regtest started ========================");
                                    println!("First bitcoin node: http://127.0.0.1:{}/wallet/default", node1_port);
                                    println!("Second bitcoin node: http://127.0.0.1:{}/wallet/default", node2_port);
                                    println!("Hexstody BTC adapter API docs: {}/rapidoc", hbtc_url);
                                    println!("Hexstody operator API docs: http://127.0.0.1:{}/swagger", op_port);
                                    println!("Hexstody public API docs: http://127.0.0.1:{}/swagger", pub_port);
                                    println!("Hexstody operator GUI: http://127.0.0.1:{}", op_port);
                                    println!("Hexstody public GUI: http://127.0.0.1:{}", pub_port);
                                }
                            };
                            join(run_fut, msg_fut).await;
                            ()
                        }
                    },
                )
                .await
            } else {
                let btc_client = BtcClient::new(&args.btc_module);
                let eth_client = EthClient::new(&args.eth_module);
                let ticker_client = TickerClient::new(&args.ticker_provider);
                let start_notify = Arc::new(Notify::new());
                run(btc_client, eth_client, ticker_client, &args, start_notify).await
            }
        }
    }
    Ok(())
}

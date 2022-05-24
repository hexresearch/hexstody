mod api;
mod runner;

use clap::Parser;
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
    /// Base64 encoded 64 byte secret key for encoding cookies. Required in release profile.
    #[clap(long, env = "HEXSTODY_SECRET_KEY", hide_env_values = true)]
    secret_key: Option<String>,
    /// Path to HTML static files to serve
    #[clap(long, env = "HEXSTODY_STATIC_PATH")]
    static_path: Option<String>,
    #[clap(long, short, env = "HEXSTODY_PORT")]
    port: Option<u16>,
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
        let default_static_path = rocket::fs::relative!("static/").to_owned();
        let static_path = args.static_path.as_ref().unwrap_or(&default_static_path);
        match run_api(
            args.network,
            args.port,
            &args.dbconnect,
            start_notify,
            args.secret_key.as_deref(),
            static_path,
        )
        .await
        {
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

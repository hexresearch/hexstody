mod api;

use clap::Parser;
use futures::future::{AbortHandle, Abortable, Aborted};
use log::*;
use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;

use hexstody_db::create_db_pool;
use hexstody_db::queries::query_state;
use hexstody_db::{state::State, Pool};

use api::operator::*;
use api::public::*;

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
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    /// Start listening incoming API requests
    Serve,
}

#[derive(Clone, Debug, PartialEq)]
enum ApiType {
    Public,
    Operator,
}

impl fmt::Display for ApiType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ApiType::Public => write!(f, "Public"),
            ApiType::Operator => write!(f, "Operator"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiConfig {
    public_api_enabled: bool,
    public_api_port: u16,
    operator_api_enabled: bool,
    operator_api_port: u16,
}

fn parse_api_config() -> ApiConfig {
    let figment = rocket::Config::figment();
    let public_api_enabled = figment.extract_inner("public_api_enabled").unwrap_or(true);
    let public_api_port = figment.extract_inner("public_api_port").unwrap_or(8000);
    let operator_api_enabled = figment
        .extract_inner("operator_api_enabled")
        .unwrap_or(true);
    let operator_api_port = figment.extract_inner("operator_api_port").unwrap_or(8001);
    ApiConfig {
        public_api_enabled,
        public_api_port,
        operator_api_enabled,
        operator_api_port,
    }
}

async fn serve_api(
    pool: Pool,
    state_mx: Arc<Mutex<State>>,
    state_notify: Arc<Notify>,
    api_type: ApiType,
    api_enabled: bool,
    port: u16,
) -> () {
    if !api_enabled {
        info!("{api_type} API disabled");
        return ();
    };
    info!("Serving {api_type} API");
    match api_type {
        ApiType::Public => loop {
            let (_abort_api_handle, abort_api_reg) = AbortHandle::new_pair();
            let api_fut = tokio::spawn(serve_public_api(
                pool.clone(),
                state_mx.clone(),
                state_notify.clone(),
                port,
            ));
            match Abortable::new(api_fut, abort_api_reg).await {
                Ok(_) => (),
                Err(Aborted) => {
                    error!("{api_type} API thread aborted")
                }
            }
            let restart_dt = Duration::from_secs(5);
            info!(
                "Adding {:?} delay before restarting {api_type} API logic",
                restart_dt
            );
            sleep(restart_dt).await;
        },
        ApiType::Operator => tokio::spawn(serve_operator_api(pool, state_mx, state_notify, port)),
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let api_config = parse_api_config();
    env_logger::init();
    match args.subcmd.clone() {
        SubCommand::Serve => {
            info!("Connecting to database");
            let pool = create_db_pool(&args.dbconnect).await?;
            info!("Connected");
            info!("Reconstructing state from database");
            let state = query_state(&pool).await?;
            let state_mx = Arc::new(Mutex::new(state));
            let state_notify = Arc::new(Notify::new());
            let public_api_fut = serve_api(
                pool.clone(),
                state_mx.clone(),
                state_notify.clone(),
                ApiType::Public,
                api_config.public_api_enabled,
                api_config.public_api_port,
            );
            let operator_api_fut = serve_api(
                pool,
                state_mx,
                state_notify,
                ApiType::Operator,
                api_config.operator_api_enabled,
                api_config.operator_api_port,
            );
            tokio::join!(public_api_fut, operator_api_fut);
        }
    }
    Ok(())
}

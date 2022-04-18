mod api;

use clap::Parser;
use futures::future::{AbortHandle, Abortable, Aborted};
use log::*;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;

use hexstody_db::create_db_pool;
use hexstody_db::queries::query_state;
use api::public::*;
use api::webserver::*;

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
    Serve {
        /// Host name to bind the service to
        #[clap(
            long,
            default_value = "0.0.0.0",
            env = "HEXSTODY_HOST"
        )]
        public_host: String,
        /// Port to bind the service to
        #[clap(long, short, default_value = "8480", env = "HEXSTODY_PORT")]
        public_port: u16,
    },
    /// Output swagger spec for public API
    SwaggerPublic,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    env_logger::init();

    match args.subcmd.clone() {
        SubCommand::Serve {
            public_host,
            public_port,
        } => loop {
            let args = args.clone();
            let (_abort_api_handle, abort_api_reg) = AbortHandle::new_pair();

            info!("Connecting to database");
            let pool = create_db_pool(&args.dbconnect).await?;
            info!("Connected");

            info!("Reconstructing state from database");
            let state = query_state(&pool).await?;
            let state_mx = Arc::new(Mutex::new(state));
            let state_notify = Arc::new(Notify::new());

            info!("Serving API");

            let public_api_fut = tokio::spawn(serve_public_api2());
            //let public_api_fut = serve_public_api(&public_host, public_port, pool, state_mx, state_notify);
            match Abortable::new(public_api_fut, abort_api_reg).await {
                Ok(mres) => mres?,
                Err(Aborted) => {
                    error!("API thread aborted");
                }
            }

            let restart_dt = Duration::from_secs(5);
            info!("Adding {:?} delay before restarting logic", restart_dt);
            sleep(restart_dt).await;
        },
        SubCommand::SwaggerPublic => {
            let pool = create_db_pool(&args.dbconnect).await?;
            let specs = public_api_specs(pool).await?;
            let specs_str = serde_json::to_string_pretty(&specs)?;
            println!("{}", specs_str);
        }
    }
    Ok(())
}

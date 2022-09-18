mod types;
mod utils;
mod db_functions;
mod node_calls;
mod handlers;
mod conf;
mod worker;

#[macro_use] extern crate rocket;

use hex_literal::hex;
use worker::*;
use conf::load_config;
use conf::NodeConfig;
use clap::Parser;

use types::*;
use utils::*;

use std::str::FromStr;

use secp256k1::SecretKey;

use web3::{
    contract::{Contract, Options},
    ethabi::ethereum_types::U256,
    types::{Address, TransactionParameters, H160},
    api::{Web3Api}
};

use std::time::Duration;
use rocket::http::{Status, ContentType};
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::{Database, Connection};
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};


use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use log::Level::*;


#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Serve {
        #[clap(
            long,
            short,
            env = "RUSTETH_CONFIG"
        )]
        config: String
    }
}

#[launch]
async fn rocket() -> _ {
    let args = Args::parse();
    match args.subcmd.clone() {
        SubCommand::Serve {
            config
        } => { println!("args: {:0}", config);}
    }
    let cfg = load_config("config.json");

    Builder::new()
        .format(|buf, record| {
            let mut level_style = buf.style();
            match record.level() {
                Error => {level_style.set_color(env_logger::fmt::Color::Red).set_bold(true);},
                Info  => {level_style.set_color(env_logger::fmt::Color::Green).set_bold(true);},
                Warn  => {level_style.set_color(env_logger::fmt::Color::Yellow).set_bold(true);},
                Debug => {level_style.set_color(env_logger::fmt::Color::Blue).set_bold(true);},
                Trace => {level_style.set_color(env_logger::fmt::Color::White).set_bold(true);},
            };
            writeln!(buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                level_style.value(record.level()),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let dbconnect = cfg.dburl.clone();
    let pool = db_functions::create_db_pool(&dbconnect).await.unwrap();



    tokio::spawn({
    let polling_duration = Duration::from_secs(cfg.api_call_timeout);
        async move {
            node_worker(polling_duration, &pool).await;
        }
    });
    rocket::build()
                    .attach(MyDb::init())
                    .manage(cfg)
                    .mount("/", openapi_get_routes![
                    handlers::common::getversion,

                    handlers::accounts::user_create,
                    handlers::accounts::user_remove,
                    handlers::accounts::user_get,
                    handlers::accounts::allocate_address,
                    handlers::accounts::check_address,
                    handlers::accounts::accounts_get,
                    handlers::accounts::tokens_get,
                    handlers::accounts::tokens_post,

                    handlers::balance::balance_eth_total,
                    handlers::balance::balance_erc20_total,
                    handlers::balance::balance_eth_login,
                    handlers::balance::balance_eth_address,
                    handlers::balance::balance_erc20_login,
                    handlers::balance::balance_erc20_address,

                    handlers::sending::send_eth_from_login,
                    handlers::sending::send_eth_from_address,
                    handlers::sending::send_erc20_from_address,
                    handlers::sending::unlock_eth,
                    handlers::sending::oldsend,
                    handlers::sending::senddummy_eth_from_address,
                    handlers::sending::senddummy_eth_from_login,
                    handlers::sending::signsend,
                    handlers::sending::signsend_erc20
                    ])
                    .mount(
                        "/swagger/",
                        make_swagger_ui(&SwaggerUIConfig {
                            url: "../openapi.json".to_owned(),
                            ..Default::default()
                        }),
                    )
}

pub mod auth;
pub mod wallet;

use auth::*;
use figment::Figment;
use log::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, Notify};

use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::uri;
use rocket::{get, routes, State};
use rocket_dyn_templates::Template;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};

use hexstody_api::domain::Currency;
use hexstody_api::error;
use hexstody_btc_client::client::BtcClient;
use hexstody_btc_client::client::BTC_BYTES_PER_TRANSACTION;
use hexstody_db::state::State as DbState;
use hexstody_db::update::*;
use hexstody_db::Pool;
use wallet::*;

#[openapi(tag = "ping")]
#[get("/ping")]
fn ping() -> Json<()> {
    Json(())
}

#[openapi(skip)]
#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(signin))
}

#[openapi(skip)]
#[get("/overview")]
fn overview() -> Template {
    let context = HashMap::from([("title", "Overview"), ("parent", "base_with_header")]);
    Template::render("overview", context)
}

#[openapi(skip)]
#[get("/tokens")]
fn tokens() -> Template {
    let context = HashMap::from([("title", "Token settings"), ("parent", "base_with_header")]);
    Template::render("tokens", context)
}

#[openapi(skip)]
#[get("/signup")]
fn signup() -> Template {
    let context = HashMap::from([("title", "Sign Up"), ("parent", "base")]);
    Template::render("signup", context)
}

#[openapi(skip)]
#[get("/signin")]
fn signin() -> Template {
    let context = HashMap::from([("title", "Sign In"), ("parent", "base")]);
    Template::render("signin", context)
}

#[openapi(skip)]
#[get("/deposit")]
fn deposit() -> Template {
    let context = HashMap::from([("title", "Deposit"), ("parent", "base_with_header")]);
    Template::render("deposit", context)
}

#[openapi(skip)]
#[get("/withdraw")]
async fn withdraw(
    btc: &State<BtcClient>,
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Template> {
    require_auth_user(cookies, state, |_, user| async move {
        let btc_fee_per_byte = &btc
            .get_fees()
            .await
            .map_err(|e| {
                error!("{}", e);
                error::Error::FailedGetFee(Currency::BTC)
            })?
            .fee_rate;

        let btc_fee_per_transaction = &(btc_fee_per_byte * BTC_BYTES_PER_TRANSACTION).to_string();
        let btc_balance = &user
            .currencies
            .get(&Currency::BTC)
            .unwrap()
            .finalized_balance()
            .to_string();

        let ethfee = &1000.to_string();
        let eth_balance = &user
            .currencies
            .get(&Currency::ETH)
            .unwrap()
            .finalized_balance()
            .to_string();
        let context = HashMap::from([
            ("title", "Withdraw"),
            ("parent", "base_with_header"),
            ("login", "lalala"),
            ("btc_balance", btc_balance),
            ("btc_fee", btc_fee_per_transaction),
            ("eth_balance", eth_balance),
            ("ethfee", ethfee),
        ]);
        Ok(Template::render("withdraw", context))
    })
    .await
}

pub async fn serve_api(
    pool: Pool,
    state: Arc<Mutex<DbState>>,
    _state_notify: Arc<Notify>,
    start_notify: Arc<Notify>,
    update_sender: mpsc::Sender<StateUpdate>,
    btc_client: BtcClient,
    api_config: Figment,
) -> Result<(), rocket::Error> {
    let on_ready = AdHoc::on_liftoff("API Start!", |_| {
        Box::pin(async move {
            start_notify.notify_one();
        })
    });
    let static_path: PathBuf = api_config.extract_inner("static_path").unwrap();
    let _ = rocket::custom(api_config)
        .mount("/", FileServer::from(static_path))
        .mount(
            "/",
            openapi_get_routes![
                ping,
                get_balance,
                get_deposit,
                get_deposit_eth,
                eth_ticker,
                btc_ticker,
                get_user_data,
                erc20_ticker,
                ethfee,
                get_history,
                get_history_eth,
                get_history_erc20,
                withdraw_eth,
                post_withdraw,
                signup_email,
                signin_email,
                logout,
                list_tokens,
                enable_token,
                disable_token
            ],
        )
        .mount(
            "/",
            routes![index, overview, tokens, signup, signin, deposit, withdraw],
        )
        .mount(
            "/swagger/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .manage(state)
        .manage(pool)
        .manage(update_sender)
        .manage(btc_client)
        .attach(Template::fairing())
        .attach(on_ready)
        .launch()
        .await?;
    Ok(())
}

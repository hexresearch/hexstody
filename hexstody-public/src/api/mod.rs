pub mod auth;
pub mod wallet;

use auth::*;
use chrono::prelude::*;
use figment::Figment;
use hexstody_api::domain::currency::Currency;
use hexstody_api::types as api;
use hexstody_api::types::History;
use hexstody_btc_client::client::BtcClient;
use hexstody_db::state::*;
use hexstody_db::update::*;
use hexstody_db::Pool;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::uri;
use rocket::{get, routes};
use rocket_dyn_templates::Template;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, Notify};
use wallet::*;

#[openapi(tag = "ping")]
#[get("/ping")]
fn ping() -> Json<()> {
    Json(())
}

#[openapi(tag = "get_history")]
#[get("/get_history/<_skip>/<_take>")]
fn get_history(_skip: u32, _take: u32) -> Json<History> {
    let x = History {
        target_number_of_confirmations: 6,
        history_items: vec![
            api::HistoryItem::Deposit(api::DepositHistoryItem {
                currency: Currency::BTC,
                date: Utc::now().naive_utc(),
                value: u64::MAX,
                number_of_confirmations: 3,
            }),
            api::HistoryItem::Withdrawal(api::WithdrawalHistoryItem {
                currency: Currency::ETH,
                date: Utc::now().naive_utc(),
                value: u64::MAX,
                status: api::WithdrawalRequestStatus::InProgress,
            }),
        ],
    };

    Json(x)
}

#[openapi(skip)]
#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(signin))
}

#[openapi(skip)]
#[get("/overview")]
fn overview() -> Template {
    let context = HashMap::from([("title", "Overview"), ("parent", "base_footer_header")]);
    Template::render("overview", context)
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
    let context = HashMap::from([("title", "Deposit"), ("parent", "base_footer_header")]);
    Template::render("deposit", context)
}

pub async fn serve_api(
    pool: Pool,
    state: Arc<Mutex<State>>,
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
                get_history,
                signup_email,
                signin_email,
                logout
            ],
        )
        .mount("/", routes![index, overview, signup, signin, deposit])
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
pub mod auth;
pub mod wallet;

use auth::*;
use figment::Figment;
use hexstody_api::domain::Currency;
use hexstody_api::error;
use hexstody_api::error::ErrorMessage;
use hexstody_btc_client::client::BtcClient;
use hexstody_db::state::State as DbState;
use hexstody_db::state::*;
use hexstody_db::update::*;
use hexstody_db::Pool;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::http::CookieJar;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::uri;
use rocket::{get, routes, State};
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

#[openapi(skip)]
#[get("/withdraw")]
async fn withdraw(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> std::result::Result<Template, (Status, Json<ErrorMessage>)> {
    let e = &u64::MAX.to_string();
    if let Some(cookie) = cookies.get_private("user_id") {
        let user_id = cookie.value();
        {
            let state = state.lock().await;
            if let Some(user) = state.users.get(user_id) {
                let btc_balance = &user
                    .currencies
                    .get(&Currency::BTC)
                    .unwrap()
                    .finalized_balance()
                    .to_string();
                let eth_balance = &user
                    .currencies
                    .get(&Currency::ETH)
                    .unwrap()
                    .finalized_balance()
                    .to_string();
                let context = HashMap::from([
                    ("title", "Withdraw"),
                    ("parent", "base_footer_header"),
                    ("btc_balance", btc_balance),
                    ("eth_balance", eth_balance),
                ]);
                Ok(Template::render("withdraw", context))
            } else {
                Err(error::Error::NoUserFound.into())
            }
        }
    } else {
        Err(error::Error::AuthRequired.into())
    }
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
                get_history,
                signup_email,
                signin_email,
                logout
            ],
        )
        .mount(
            "/",
            routes![index, overview, signup, signin, deposit, withdraw],
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

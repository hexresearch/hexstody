use figment::Figment;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, Notify};
use wallet::*;

use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::uri;
use rocket::{get, routes, State};
use rocket_dyn_templates::Template;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};


use hexstody_api::error::{self, ErrorMessage};
use hexstody_btc_client::client::BtcClient;
use hexstody_db::state::State as DbState;
use hexstody_db::update::*;
use hexstody_db::Pool;

pub mod auth;
pub mod wallet;
use auth::*;

/// Redirect to signin page
fn goto_signin() -> Redirect {
    Redirect::to(uri!(signin))
}

#[openapi(tag = "ping")]
#[get("/ping")]
fn ping() -> Json<()> {
    Json(())
}

#[openapi(skip)]
#[get("/")]
async fn index(cookies: &CookieJar<'_>) -> Redirect {
    require_auth(cookies, |_| async { Ok(()) })
        .await
        .map_or(goto_signin(), |_| Redirect::to(uri!(overview)))
}

#[openapi(skip)]
#[get("/overview")]
async fn overview(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let context = HashMap::from([
            ("title", "Overview"),
            ("username", &user.username),
            ("parent", "base_with_header"),
        ]);
        Ok(Template::render("overview", context))
    })
    .await
    .map_err(|_| goto_signin())
}

#[openapi(skip)]
#[get("/profile")]
async fn profile(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let context = HashMap::from([
            ("title", "Profile"),
            ("username", &user.username),
            ("parent", "base_with_header"),
        ]);
        Ok(Template::render("profile", context))
    })
    .await
    .map_err(|_| goto_signin())
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

#[openapi(tag = "auth")]
#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Result<Redirect, (Status, Json<ErrorMessage>)> {
    let resp = require_auth(cookies, |cookie| async move {
        cookies.remove(cookie);
        Ok(Json(()))
    })
    .await;
    match resp {
        Ok(_) => Ok(goto_signin()),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => {
            if err.1.code == 8 || err.1.code == 7 {
                Ok(goto_signin())
            } else {
                Err(err)
            }
        }
    }
}

#[openapi(skip)]
#[get("/deposit")]
async fn deposit(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let context = HashMap::from([
            ("title", "Deposit"),
            ("username", &user.username),
            ("parent", "base_with_header"),
        ]);
        Ok(Template::render("deposit", context))
    })
    .await
    .map_err(|_| goto_signin())
}

#[openapi(skip)]
#[get("/withdraw")]
async fn withdraw(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Template> {
    require_auth_user(cookies, state, |_, _| async move {
        let context = json!({
            "title" : "Withdraw",
            "parent": "base_footer_header",
            "tabs"   : ["btc", "eth"]}
        );
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
            routes![index, overview, profile, signup, signin, deposit, withdraw],
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

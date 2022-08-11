pub mod auth;
pub mod wallet;

use auth::*;
use figment::Figment;
use hexstody_api::domain::Currency;
use hexstody_api::types::{LimitApiResp, LimitChangeReq, LimitChangeData, LimitChangeResponse};
use hexstody_db::update::misc::{LimitCancelData, LimitChangeUpd};
use hexstody_eth_client::client::EthClient;
use rocket::http::hyper::request;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, Notify};

use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::uri;
use rocket::{get, post, routes, State};
use rocket_dyn_templates::Template;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};

use hexstody_api::error::{self, ErrorMessage};
use hexstody_btc_client::client::BtcClient;
use hexstody_db::state::State as DbState;
use hexstody_db::update::*;
use hexstody_db::Pool;
use wallet::*;

/// Redirect to signin page
fn goto_signin() -> Redirect{
    Redirect::to(uri!(signin))
}

#[openapi(tag = "ping")]
#[get("/ping")]
fn ping() -> Json<()> {
    Json(())
}

#[openapi(skip)]
#[get("/")]
async fn index(
    cookies: &CookieJar<'_>,
) -> Redirect {
    require_auth(cookies, |_| async {Ok(())})
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
        let context = HashMap::from([("title", "Overview"), ("username", &user.username), ("parent", "base_with_header")]);
        Ok(Template::render("overview", context))
    }).await.map_err(|_| goto_signin())
}

#[openapi(skip)]
#[get("/profile")]
async fn profile(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let context = json!({
            "title" : "Profile",
            "parent": "base_footer_header",
            "tabs"   : ["tokens", "limits"],
            "username": &user.username
        }
        );
        Ok(Template::render("profile", context))
    }).await.map_err(|_| goto_signin())
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
pub async fn logout(cookies: &CookieJar<'_>) 
-> Result<Redirect, (Status, Json<ErrorMessage>)> {
    let resp = require_auth(cookies, |cookie| async move {
        cookies.remove(cookie);
        Ok(Json(()))
    }).await;
    match resp {
        Ok(_) => Ok(goto_signin()),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => if err.1.code == 8 || err.1.code == 7 {
            Ok(goto_signin())
        } else {
            Err(err)
        },
    }
}

#[openapi(skip)]
#[get("/deposit")]
async fn deposit(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let context = HashMap::from([("title", "Deposit"), ("username", &user.username), ("parent", "base_with_header")]);
        Ok(Template::render("deposit", context))
    }).await.map_err(|_| goto_signin())
}

#[openapi(skip)]
#[get("/withdraw")]
async fn withdraw(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<error::Result<Template>, Redirect> {
    let resp = require_auth_user(cookies, state, |_, user| async move {
        let context = json!({
            "title" : "Withdraw",
            "parent": "base_footer_header",
            "tabs"   : ["btc", "eth"],
            "username": &user.username
        }
        );
        Ok(Template::render("withdraw", context))
    }).await;
    match resp {
        Ok(v) => Ok(Ok(v)),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => if err.1.code == 8 || err.1.code == 7 {
            Err(goto_signin())
        } else {
            Ok(Err(err))
        },
    }
}

#[openapi(skip)]
#[get("/removeuser/<user>")]
pub async fn remove_user(
    eth_client: &State<EthClient>,
    state: &State<Arc<Mutex<hexstody_db::state::State>>>,
    is_test: &State<IsTestFlag>,
    user: &str
) -> Result<(), Redirect> {
    if is_test.0 {
        let _ = eth_client.remove_user(&user).await;
        let mut mstate = state.lock().await;
        mstate.users.remove(user);
        Ok(())
    } else {
        Err(Redirect::to(uri!(signin)))
    }

}

#[openapi(skip)]
#[get("/profile/limits/get")]
pub async fn get_user_limits(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<Json<Vec<LimitApiResp>>, Redirect>{
    require_auth_user(cookies, state, |_, user| async move {
        let infos = user.currencies.values().map(|cur_info| 
            LimitApiResp{ 
                limit_info: cur_info.limit_info.clone(), 
                currency: cur_info.currency.clone() 
            }).collect();
        Ok(Json(infos))
    }).await.map_err(|_| goto_signin())
}

#[openapi(skip)]
#[post("/profile/limits", data="<new_limits>")]
pub async fn request_new_limits(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    new_limits: Json<Vec<LimitChangeReq>>
) -> Result<error::Result<()>, Redirect> {
    let new_limits = new_limits.into_inner();
    let resp = require_auth_user(cookies, state, |_, user| async move {
        let filtered_limits : Vec<LimitChangeUpd> = new_limits.into_iter().filter_map(|l| {
            match user.currencies.get(&l.currency) {
                None => None,
                Some(ci) => if ci.limit_info.limit == l.limit{
                    None
                } else {
                    Some(LimitChangeUpd{
                        user: user.username.clone(),
                        currency: l.currency.clone(),
                        limit: l.limit.clone(),
                    })
                }
            }
        }).collect();
        if filtered_limits.is_empty(){
            Err(error::Error::InviteNotFound.into())
        } else {
           for req in filtered_limits {
            let state_update = StateUpdate::new(UpdateBody::LimitsChangeRequest(req));
            let _ = updater.send(state_update).await;
            }
            Ok(())
        }
    }).await;
    match resp {
        Ok(v) => Ok(Ok(v)),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => if err.1.code == 8 || err.1.code == 7 {
            Err(goto_signin())
        } else {
            Ok(Err(err))
        },
    }
}

#[openapi(skip)]
#[get("/profile/limits/changes")]
pub async fn get_user_limit_changes(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<Json<Vec<LimitChangeResponse>>, Redirect>{
    require_auth_user(cookies, state, |_, user| async move {
        let changes = user.limit_change_requests.values().map(|v| {
            let LimitChangeData{ id, user, created_at, status, currency, limit, .. } = v.clone();
            let request = LimitChangeReq{currency, limit};
            LimitChangeResponse{ id, user, created_at, request,status}
        }).collect();
        Ok(Json(changes))
    }).await.map_err(|_| goto_signin())
}

#[openapi(skip)]
#[post("/profile/limits/cancel", data="<currency>")]
pub async fn cancel_user_change(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    currency: Json<Currency>
) -> Result<error::Result<()>, Redirect>{
    let resp = require_auth_user(cookies, state, |_, user| async move {
        match user.limit_change_requests.get(&currency){
            Some(v) => {
                let state_update = StateUpdate::new(UpdateBody::CancelLimitChange(
                    LimitCancelData{ id: v.id.clone(), user: user.username.clone(), currency: currency.into_inner().clone() }));
                let _ = updater.send(state_update).await;
                Ok(())
            },
            None => return Err(error::Error::LimChangeNotFound.into()),
        }
    }).await;
    match resp {
        Ok(v) => Ok(Ok(v)),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => if err.1.code == 8 || err.1.code == 7 {
            Err(goto_signin())
        } else {
            Ok(Err(err))
        },
    }
}
pub async fn serve_api(
    pool: Pool,
    state: Arc<Mutex<DbState>>,
    _state_notify: Arc<Notify>,
    start_notify: Arc<Notify>,
    update_sender: mpsc::Sender<StateUpdate>,
    btc_client: BtcClient,
    eth_client: EthClient,
    api_config: Figment,
    is_test: bool
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
                ticker,
                get_user_data,
                erc20_ticker,
                ethfee,
                btcfee,
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
                disable_token,
                remove_user,
                get_user_limits,
                request_new_limits,
                get_user_limit_changes,
                cancel_user_change
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
        .manage(eth_client)
        .manage(IsTestFlag(is_test))
        .attach(Template::fairing())
        .attach(on_ready)
        .launch()
        .await?;
    Ok(())
}

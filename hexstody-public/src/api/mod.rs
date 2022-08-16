pub mod auth;
pub mod wallet;
pub mod profile;
pub mod helpers;

use hexstody_api::domain::Language;
use profile::*;
use auth::*;
use figment::Figment;
use hexstody_eth_client::client::EthClient;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, Notify};

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
use wallet::*;

struct StaticPath(PathBuf);

fn get_dict_json(
    static_path: &StaticPath, 
    lang: Language, 
    path: PathBuf
) -> error::Result<serde_json::Value>{
    let file_path = format!("{}/lang/{}/{}", static_path.0.display(), lang.to_alpha(), path.display());
    let file = File::open(file_path);
    if let Err(e) = file { 
        return Err(error::Error::GenericError(format!("Failed to open file: {:?}", e)).into())
    };
    let mut file = file.unwrap();
    let mut data = String::new();
    if let Err(e) = file.read_to_string(&mut data){
       return Err(error::Error::GenericError(format!("Failed to read file: {:?}", e)).into()) 
    };
    Ok(serde_json::from_str(&data).unwrap())
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
    static_path: &State<StaticPath>
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let title = match user.config.language{
            Language::English => "Overview",
            Language::Russian => "Главная",
        };
        let header_dict = get_dict_json(static_path.inner(), user.config.language, PathBuf::from_str("header.json").unwrap());
        let overview_dict = get_dict_json(static_path.inner(), user.config.language, PathBuf::from_str("overview.json").unwrap());
        if let Err(e) = header_dict { return Err(e) };
        if let Err(e) = overview_dict { return Err(e) };
        let context = json!({
            "title":title, 
            "username": &user.username, 
            "parent": "base_with_header",
            "lang": json!({
                "lang": user.config.language.to_alpha().to_uppercase(),
                "header": header_dict.unwrap(),
                "overview": overview_dict.unwrap()
            })
        });
        Ok(Template::render("overview", context))
    }).await.map_err(|_| goto_signin())
}

#[openapi(skip)]
#[get("/profile?<tab>")]
async fn profile_page(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    static_path: &State<StaticPath>,
    tab: Option<String>
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let title = match user.config.language{
            Language::English => "Profile",
            Language::Russian => "Профиль",
        };
        let tabs = match user.config.language {
            Language::English =>  [
                json!({"id": "tokens", "label": "tokens"}), 
                json!({"id": "limits", "label": "limits"}),
                json!({"id": "settings", "label": "settings"})],
            Language::Russian => [
                json!({"id": "tokens", "label": "токены"}), 
                json!({"id": "limits", "label": "лимиты"}),
                json!({"id": "settings", "label": "настройки"})],
        };
        let header_dict = get_dict_json(static_path.inner(), user.config.language, PathBuf::from_str("header.json").unwrap());
        if let Err(e) = header_dict { return Err(e) };
        let context = json!({
            "title" : title,
            "parent": "base_footer_header",
            "tabs"  : tabs,
            "selected": tab.unwrap_or("tokens".to_string()),
            "username": &user.username,
            "lang": json!({
                "lang": user.config.language.to_alpha().to_uppercase(),
                "header": header_dict.unwrap(),
            })
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
    static_path: &State<StaticPath>
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let title = match user.config.language{
            Language::English => "Deposit",
            Language::Russian => "Депозит",
        };
        let header_dict = get_dict_json(static_path.inner(), user.config.language, PathBuf::from_str("header.json").unwrap());
        if let Err(e) = header_dict { return Err(e) };
        let context = json!({
            "title" : title,
            "parent": "base_footer_header",
            "username": &user.username,
            "lang": json!({
                "lang": user.config.language.to_alpha().to_uppercase(),
                "header": header_dict.unwrap(),
            })
        });
        Ok(Template::render("deposit", context))
    }).await.map_err(|_| goto_signin())
}

#[openapi(skip)]
#[get("/withdraw")]
async fn withdraw(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    static_path: &State<StaticPath>
) -> Result<error::Result<Template>, Redirect> {
    let resp = require_auth_user(cookies, state, |_, user| async move {
        let title = match user.config.language{
            Language::English => "Withdraw",
            Language::Russian => "Вывод",
        };
        let header_dict = get_dict_json(static_path.inner(), user.config.language, PathBuf::from_str("header.json").unwrap());
        if let Err(e) = header_dict { return Err(e) };
        let context = json!({
            "title" : title,
            "parent": "base_footer_header",
            "tabs"   : ["btc", "eth"],
            "username": &user.username,
            "lang": json!({
                "lang": user.config.language.to_alpha().to_uppercase(),
                "header": header_dict.unwrap(),
            })
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
#[get("/lang/<path..>")]
async fn get_dict(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    static_path: &State<StaticPath>,
    path: PathBuf
) -> error::Result<serde_json::Value> {
    require_auth_user(cookies, state, |_, user| async move {
        get_dict_json(static_path.inner(), user.config.language, path)
    }).await
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
        .mount("/", FileServer::from(static_path.clone()))
        .mount(
            "/",
            openapi_get_routes![
                ping,
                get_balance,
                get_balance_by_currency,
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
                cancel_user_change,
                set_language,
                get_dict
            ],
        )
        .mount(
            "/",
            routes![index, overview, profile_page, signup, signin_page, deposit, withdraw],
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
        .manage(StaticPath(static_path))
        .attach(Template::custom(|engine|{
            engine.handlebars.register_helper("isEqString", Box::new(helpers::is_eq_string))
        }))
        .attach(on_ready)
        .launch()
        .await?;
    Ok(())
}

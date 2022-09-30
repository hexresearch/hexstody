pub mod auth;
pub mod helpers;
pub mod profile;
pub mod wallet;

use base64;
use figment::Figment;
use hexstody_db::state::Network;
use hexstody_runtime_db::RuntimeState;
use hexstody_ticker::api::ticker_api;
use hexstody_ticker_provider::client::TickerClient;
use qrcode_generator::QrCodeEcc;
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
use rocket_dyn_templates::{context, Template};
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};

use auth::*;
use hexstody_api::{
    domain::{Currency, Language},
    error::{self, ErrorMessage},
    types::DepositInfo,
};
use hexstody_btc_client::client::BtcClient;
use hexstody_db::{state::State as DbState, update::*, Pool};
use hexstody_eth_client::client::EthClient;
use hexstody_sig::SignatureVerificationConfig;
use profile::*;
use wallet::*;

struct StaticPath(PathBuf);

fn get_dict_json(
    static_path: &StaticPath,
    lang: Language,
    path: PathBuf,
) -> error::Result<serde_json::Value> {
    let file_path = format!(
        "{}/lang/{}/{}",
        static_path.0.display(),
        lang.to_alpha(),
        path.display()
    );
    let file = File::open(file_path);
    if let Err(e) = file {
        return Err(error::Error::GenericError(format!("Failed to open file: {:?}", e)).into());
    };
    let mut file = file.unwrap();
    let mut data = String::new();
    if let Err(e) = file.read_to_string(&mut data) {
        return Err(error::Error::GenericError(format!("Failed to read file: {:?}", e)).into());
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
    static_path: &State<StaticPath>,
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let page_title = match user.config.language {
            Language::English => "Overview",
            Language::Russian => "Главная",
        };
        let header_dict = get_dict_json(
            static_path.inner(),
            user.config.language,
            PathBuf::from_str("header.json").unwrap(),
        );
        let overview_dict = get_dict_json(
            static_path.inner(),
            user.config.language,
            PathBuf::from_str("overview.json").unwrap(),
        );
        if let Err(e) = header_dict {
            return Err(e);
        };
        if let Err(e) = overview_dict {
            return Err(e);
        };
        let context = context! {
            page_title,
            parent: "base_with_header",
            username: &user.username,
            lang: context! {
                selected_lang: user.config.language.to_alpha().to_uppercase(),
                header: header_dict.unwrap(),
                overview: overview_dict.unwrap()
            }
        };
        Ok(Template::render("overview", context))
    })
    .await
    .map_err(|_| goto_signin())
}

#[openapi(skip)]
#[get("/profile?<tab>")]
async fn profile_page(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    static_path: &State<StaticPath>,
    tab: Option<String>,
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let page_title = match user.config.language {
            Language::English => "Profile",
            Language::Russian => "Профиль",
        };
        let tabs = get_dict_json(
            static_path.inner(),
            user.config.language,
            PathBuf::from_str("profile-tabs.json").unwrap(),
        );
        if let Err(e) = tabs {
            return Err(e);
        };
        let header_dict = get_dict_json(
            static_path.inner(),
            user.config.language,
            PathBuf::from_str("header.json").unwrap(),
        );
        if let Err(e) = header_dict {
            return Err(e);
        };
        let context = context! {
            page_title,
            parent: "base_with_header",
            tabs: tabs.unwrap(),
            selected: tab.unwrap_or("tokens".to_string()),
            username: &user.username,
            lang: context! {
                selected_lang: user.config.language.to_alpha().to_uppercase(),
                header: header_dict.unwrap(),
            }
        };
        Ok(Template::render("profile", context))
    })
    .await
    .map_err(|_| goto_signin())
}

#[openapi(skip)]
#[get("/signup")]
fn signup() -> Template {
    let context = context! {};
    Template::render("signup", context)
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
#[get("/deposit?<tab>")]
async fn deposit(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    static_path: &State<StaticPath>,
    btc_client: &State<BtcClient>,
    eth_client: &State<EthClient>,
    update_sender: &State<mpsc::Sender<StateUpdate>>,
    tab: Option<String>,
) -> Result<error::Result<Template>, Redirect> {
    let resp = require_auth_user(cookies, state, |_, user| async move {
        let page_title = match user.config.language {
            Language::English => "Deposit",
            Language::Russian => "Депозит",
        };
        let header_dict = get_dict_json(
            static_path.inner(),
            user.config.language,
            PathBuf::from_str("header.json").unwrap(),
        )?;
        let deposit_dict = get_dict_json(
            static_path.inner(),
            user.config.language,
            PathBuf::from_str("deposit.json").unwrap(),
        )?;
        let user_currencies: Vec<Currency> = user.currencies.keys().cloned().collect();
        let tabs: Vec<String> = user_currencies
            .iter()
            .map(|c| c.ticker_lowercase())
            .collect();
        let selected_tab = match tab {
            None => user_currencies[0].ticker_lowercase(),
            Some(t) => {
                if tabs.contains(&t) {
                    t
                } else {
                    user_currencies[0].ticker_lowercase()
                }
            }
        };
        let mut deposit_addresses: Vec<DepositInfo> = vec![];
        for user_currency in user_currencies.iter() {
            let deposit_address = get_deposit_address(
                btc_client,
                eth_client,
                update_sender,
                state,
                &user.username,
                user_currency.clone(),
            )
            .await?;
            let qr_code: Vec<u8> =
                qrcode_generator::to_png_to_vec(deposit_address.address(), QrCodeEcc::Low, 256)
                    .unwrap();
            deposit_addresses.push(DepositInfo {
                address: deposit_address.address(),
                qr_code_base64: base64::encode(qr_code),
                tab: user_currency.ticker_lowercase(),
                currency: user_currency.to_string(),
            });
        }
        let context = context! {
            page_title,
            parent: "base_with_header",
            deposit_addresses,
            tabs,
            selected: selected_tab,
            username: &user.username,
            lang: context! {
                selected_lang: user.config.language.to_alpha().to_uppercase(),
                header: header_dict,
                deposit: deposit_dict,
            }
        };
        Ok(Template::render("deposit", context))
    })
    .await;
    match resp {
        Ok(v) => Ok(Ok(v)),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => {
            if err.1.code == 8 || err.1.code == 7 {
                Err(goto_signin())
            } else {
                Ok(Err(err))
            }
        }
    }
}

#[openapi(skip)]
#[get("/withdraw?<tab>")]
async fn withdraw(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    static_path: &State<StaticPath>,
    tab: Option<String>,
) -> Result<error::Result<Template>, Redirect> {
    let resp = require_auth_user(cookies, state, |_, user| async move {
        let page_title = match user.config.language {
            Language::English => "Withdraw",
            Language::Russian => "Вывод",
        };
        let header_dict = get_dict_json(
            static_path.inner(),
            user.config.language,
            PathBuf::from_str("header.json").unwrap(),
        )?;
        let withdraw_dict = get_dict_json(
            static_path.inner(),
            user.config.language,
            PathBuf::from_str("withdraw.json").unwrap(),
        )?;
        let user_currencies: Vec<Currency> = user.currencies.keys().cloned().collect();
        let tabs: Vec<String> = user_currencies
            .iter()
            .map(|c| c.ticker_lowercase())
            .collect();
        let selected_tab = match tab {
            None => user_currencies[0].ticker_lowercase(),
            Some(t) => {
                if tabs.contains(&t) {
                    t
                } else {
                    user_currencies[0].ticker_lowercase()
                }
            }
        };
        let context = context! {
            page_title,
            parent: "base_with_header",
            tabs,
            selected: selected_tab,
            username: &user.username,
            lang: context! {
                selected_lang: user.config.language.to_alpha().to_uppercase(),
                header: header_dict,
                withdraw: withdraw_dict,
            }
        };
        Ok(Template::render("withdraw", context))
    })
    .await;
    match resp {
        Ok(v) => Ok(Ok(v)),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => {
            if err.1.code == 8 || err.1.code == 7 {
                Err(goto_signin())
            } else {
                Ok(Err(err))
            }
        }
    }
}

#[openapi(skip)]
#[get("/translations/<path..>")]
async fn get_dict(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    static_path: &State<StaticPath>,
    path: PathBuf,
) -> error::Result<serde_json::Value> {
    require_auth_user(cookies, state, |_, user| async move {
        get_dict_json(static_path.inner(), user.config.language, path)
    })
    .await
}

#[openapi(skip)]
#[get("/swap")]
async fn swap(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    static_path: &State<StaticPath>,
) -> Result<Template, Redirect> {
    require_auth_user(cookies, state, |_, user| async move {
        let header_dict = get_dict_json(
            static_path.inner(),
            user.config.language,
            PathBuf::from_str("header.json").unwrap(),
        )?;
        let context = context! {
            title:"swap",
            parent: "base_with_header",
            username: &user.username,
            lang: context! {
                lang: user.config.language.to_alpha().to_uppercase(),
                header: header_dict,
            }
        };
        Ok(Template::render("swap", context))
    })
    .await
    .map_err(|_| goto_signin())
}

pub async fn serve_api(
    pool: Pool,
    state: Arc<Mutex<DbState>>,
    runtime_state: Arc<Mutex<RuntimeState>>,
    _state_notify: Arc<Notify>,
    start_notify: Arc<Notify>,
    update_sender: mpsc::Sender<StateUpdate>,
    btc_client: BtcClient,
    eth_client: EthClient,
    ticker_client: TickerClient,
    api_config: Figment,
    is_test: bool,
) -> Result<(), rocket::Error> {
    let on_ready = AdHoc::on_liftoff("API Start!", |_| {
        Box::pin(async move {
            start_notify.notify_one();
        })
    });
    let static_path: PathBuf = api_config.extract_inner("static_path").unwrap();
    let network: Network = api_config.extract_inner("network").unwrap();
    let ticker_api = ticker_api();
    let _ = rocket::custom(api_config)
        .mount("/", FileServer::from(static_path.clone()))
        .mount(
            "/",
            openapi_get_routes![
                ping,
                get_balance,
                get_balance_by_currency,
                get_user_data,
                ethfee,
                btcfee,
                get_history,
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
                get_dict,
                get_user_config,
                set_user_config,
                change_password,
                set_user_public_key,
                get_challenge,
                redeem_challenge,
                get_deposit_address_handle,
                order_exchange,
                list_my_orders,
                get_network
            ],
        )
        .mount("/ticker/", ticker_api)
        .mount(
            "/",
            routes![
                index,
                overview,
                profile_page,
                signup,
                signin_page,
                deposit,
                withdraw,
                swap
            ],
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
        .manage(runtime_state)
        .manage(ticker_client)
        .manage(IsTestFlag(is_test))
        .manage(StaticPath(static_path))
        .manage(network)
        .attach(Template::fairing())
        .attach(AdHoc::config::<SignatureVerificationConfig>())
        .attach(on_ready)
        .launch()
        .await?;
    Ok(())
}

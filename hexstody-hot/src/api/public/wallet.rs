use super::auth::require_auth;
use hexstody_api::domain::currency::Currency;
use hexstody_api::error;
use hexstody_api::types as api;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use rocket_okapi::openapi;
use std::sync::Arc;
use tokio::sync::Mutex;
use hexstody_db::state::State as DbState;

#[openapi(tag = "wallet")]
#[get("/balance")]
pub async fn get_balance(cookies: &CookieJar<'_>) -> error::Result<api::Balance> {
    require_auth(cookies, |_| async move {
        let x = api::Balance {
            balances: vec![
                api::BalanceItem {
                    currency: Currency::BTC,
                    value: u64::MAX,
                },
                api::BalanceItem {
                    currency: Currency::ETH,
                    value: u64::MAX,
                },
            ],
        };
        Ok(Json(x))
    })
    .await
}

#[openapi(tag = "wallet")]
#[post("/deposit", data = "<currency>")]
pub async fn get_deposit(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    currency: Json<Currency>,
) -> error::Result<api::DepositInfo> {
    require_auth(cookies, |cookie| async move {
        let user_id = cookie.value();
        {
            let state = state.lock().await;
            if let Some(user) = state.users.get(user_id) {
                if let Some(info) = user.currencies.get(&currency.0) {
                    if let Some(address) = info.deposit_info.last() {
                        Ok(Json(api::DepositInfo {
                            address: format!("{}", address),
                        }))
                    } else {
                        // generate addresses
                        todo!();
                    }
                } else {
                    Err(error::Error::NoUserCurrency(currency.0).into())
                }
            } else {
                Err(error::Error::NoUserFound.into())
            }
        }
    })
    .await
}

use super::auth::require_auth;
use hexstody_api::domain::currency::Currency;
use hexstody_api::error;
use hexstody_api::types as api;
use rocket::get;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

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

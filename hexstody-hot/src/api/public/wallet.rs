use super::auth::require_auth;
use hexstody_api::domain::{BtcAddress, Currency, CurrencyAddress};
use hexstody_api::error::{self, ErrorMessage};
use hexstody_api::types as api;
use hexstody_btc_client::client::BtcClient;
use hexstody_db::state::State as DbState;
use hexstody_db::update::deposit::DepositAddress;
use hexstody_db::update::{StateUpdate, UpdateBody};

use log::*;
use rocket::http::CookieJar;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use rocket_okapi::openapi;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

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
    updater: &State<mpsc::Sender<StateUpdate>>,
    btc: &State<BtcClient>,
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
                        info!("Allocating new {} address for user {}", currency.0, user_id);
                        let address = allocate_address(btc, updater, user_id, currency.0).await?;

                        Ok(Json(api::DepositInfo {
                            address: format!("{}", address),
                        }))
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

async fn allocate_address(
    btc: &State<BtcClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    user_id: &str,
    currency: Currency,
) -> Result<CurrencyAddress, error::Error> {
    match currency {
        Currency::BTC => allocate_btc_address(btc, updater, user_id).await,
        Currency::ETH => todo!("Generation of addresses for ETH"),
        Currency::ERC20(_) => todo!("Generation of addresses for ETH"),
    }
}

async fn allocate_btc_address(
    btc: &State<BtcClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    user_id: &str,
) -> Result<CurrencyAddress, error::Error> {
    let address = btc.deposit_address().await.map_err(|e| {
        error!("{}", e);
        error::Error::FailedGenAddress(Currency::BTC)
    })?;

    let packed_address = CurrencyAddress::BTC(BtcAddress(format!("{}", address)));

    updater
        .send(StateUpdate::new(UpdateBody::DepositAddress(
            DepositAddress {
                user_id: user_id.to_owned(),
                address: packed_address.clone(),
            },
        )))
        .await
        .unwrap();

    Ok(packed_address)
}

use log::*;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use rocket_okapi::openapi;

use super::auth::require_auth;
use hexstody_api::domain::{BtcAddress, Currency, CurrencyAddress};
use hexstody_api::error;
use hexstody_api::types::History;
use hexstody_api::types::{self as api};
use hexstody_btc_client::client::BtcClient;
use hexstody_db::state::State as DbState;
use hexstody_db::state::Transaction as Tx;
use hexstody_db::state::WithdrawalRequest;
use hexstody_db::state::WithdrawalRequestStatus::Confirmations;
use hexstody_db::state::REQUIRED_NUMBER_OF_CONFIRMATIONS;
use hexstody_db::update::deposit::DepositAddress;
use hexstody_db::update::{StateUpdate, UpdateBody};

#[openapi(tag = "wallet")]
#[get("/balance")]
pub async fn get_balance(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<api::Balance> {
    require_auth(cookies, |cookie| async move {
        let user_id = cookie.value();
        {
            let state = state.lock().await;
            if let Some(user) = state.users.get(user_id) {
                let balances: Vec<_> = user
                    .currencies
                    .iter()
                    .map(|(cur, info)| api::BalanceItem {
                        currency: cur.clone(),
                        value: info.balance(),
                    })
                    .collect();
                Ok(Json(api::Balance { balances }))
            } else {
                Err(error::Error::NoUserFound.into())
            }
        }
    })
    .await
}

#[openapi(tag = "history")]
#[get("/history/<skip>/<take>")]
pub async fn get_history(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    skip: usize,
    take: usize,
) -> error::Result<api::History> {
    fn to_deposit_history_item(deposit: &Tx) -> api::HistoryItem {
        match deposit {
            Tx::Btc(btc_deposit) => api::HistoryItem::Deposit(api::DepositHistoryItem {
                currency: Currency::BTC,
                date: btc_deposit.timestamp,
                number_of_confirmations: btc_deposit.confirmations,
                value: btc_deposit.amount,
            }),
            Tx::Eth() => todo!("Eth deposit history mapping"),
        }
    }

    fn to_withdrawal_history_item(
        currency: &Currency,
        withdrawal: &WithdrawalRequest,
    ) -> api::HistoryItem {
        let withdrawal_status = match &withdrawal.confirmation_status {
            Confirmations(confirmations) if confirmations.len() < 6 => {
                api::WithdrawalRequestStatus::InProgress
            }
            _ => api::WithdrawalRequestStatus::Completed,
        };

        api::HistoryItem::Withdrawal(api::WithdrawalHistoryItem {
            currency: currency.to_owned(),
            date: withdrawal.created_at,
            status: withdrawal_status,
            value: withdrawal.amount,
        })
    }
    require_auth(cookies, |cookie| async move {
        let user_id = cookie.value();
        {
            let state = state.lock().await;

            if let Some(user) = state.users.get(user_id) {
                let mut history = user
                    .currencies
                    .iter()
                    .flat_map(|(currency, info)| {
                        let deposits = info.unconfirmed_transactions();
                        let deposit_history = deposits.iter().map(to_deposit_history_item);
                        let withdrawals = info.withdrawal_requests();
                        let withdrawal_history = withdrawals
                            .iter()
                            .map(|withdrawal| to_withdrawal_history_item(currency, withdrawal));

                        withdrawal_history
                            .chain(deposit_history)
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                history.sort_by(|a, b| api::history_item_time(b).cmp(api::history_item_time(a)));

                let history_slice = history[skip - 1..take].to_vec();

                Ok(Json(History {
                    target_number_of_confirmations: REQUIRED_NUMBER_OF_CONFIRMATIONS,
                    history_items: history_slice,
                }))
            } else {
                Err(error::Error::NoUserFound.into())
            }
        }
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

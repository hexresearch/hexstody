use log::*;
use reqwest;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use rocket_okapi::openapi;

use super::auth::{require_auth, require_auth_user};
use hexstody_api::domain::{BtcAddress, Currency, CurrencyAddress};
use hexstody_api::error;
use hexstody_api::types as api;
use hexstody_btc_client::client::{BtcClient, BTC_BYTES_PER_TRANSACTION};
use hexstody_db::state::State as DbState;
use hexstody_db::state::{Transaction, WithdrawalRequest, REQUIRED_NUMBER_OF_CONFIRMATIONS};
use hexstody_db::update::deposit::DepositAddress;
use hexstody_db::update::withdrawal::WithdrawalRequestInfo;
use hexstody_db::update::{StateUpdate, UpdateBody};

#[openapi(tag = "wallet")]
#[get("/balance")]
pub async fn get_balance(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Json<api::Balance>> {
    require_auth_user(cookies, state, |_, user| async move {
        let user_data =
            reqwest::get("http://node.desolator.net/userdata/".to_owned() + &user.username)
                .await
                .unwrap()
                .json::<api::UserEth>()
                .await
                .unwrap();

        let balances: Vec<api::BalanceItem> = user
            .currencies
            .iter()
            .map(|(cur, info)| {
                let mut bal = info.balance();
                match cur {
                    Currency::BTC => {}
                    Currency::ETH => {
                        bal = user_data.data.balanceEth.parse::<u64>().unwrap();
                    }
                    Currency::ERC20(token) => {
                        for tok in &user_data.data.balanceTokens {
                            if tok.tokenName == token.ticker {
                                bal = tok.tokenBalance.parse::<u64>().unwrap();
                            }
                        }
                    }
                }
                api::BalanceItem {
                    currency: cur.clone(),
                    value: bal,
                }
            })
            .collect();
        Ok(Json(api::Balance { balances }))
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
) -> error::Result<Json<api::DepositInfo>> {
    require_auth_user(cookies, state, |_, user| async move {
        if let Some(info) = user.currencies.get(&currency.0) {
            if let Some(address) = info.deposit_info.last() {
                Ok(Json(api::DepositInfo {
                    address: format!("{}", address),
                }))
            } else {
                info!(
                    "Allocating new {} address for user {}",
                    currency.0, user.username
                );
                let address = allocate_address(btc, updater, &user.username, currency.0).await?;

                Ok(Json(api::DepositInfo {
                    address: format!("{}", address),
                }))
            }
        } else {
            Err(error::Error::NoUserCurrency(currency.0).into())
        }
    })
    .await
}

#[openapi(tag = "wallet")]
#[post("/deposit_eth")]
pub async fn get_deposit_eth(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Json<api::DepositInfo>> {
    require_auth_user(cookies, state, |_, user| async move {
        let user_data_str =
            reqwest::get("http://node.desolator.net/userdata/".to_owned() + &user.username)
                .await
                .unwrap()
                .text()
                .await
                .unwrap();

        let user_data: api::UserEth = (serde_json::from_str(&user_data_str)).unwrap();
        Ok(Json(api::DepositInfo {
            address: format!("{}", &user_data.address),
        }))
    })
    .await
}

#[openapi(tag = "wallet")]
#[post("/ethticker")]
pub async fn eth_ticker(cookies: &CookieJar<'_>) -> error::Result<Json<api::TickerETH>> {
    require_auth(cookies, |_| async move {
        let ticker_eth_str =
            reqwest::get("https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=USD,RUB")
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
        let ticker_eth: api::TickerETH = (serde_json::from_str(&ticker_eth_str)).unwrap();
        Ok(Json(ticker_eth))
    })
    .await
}

#[openapi(tag = "wallet")]
#[post("/btcticker")]
pub async fn btc_ticker(cookies: &CookieJar<'_>) -> error::Result<Json<api::TickerETH>> {
    require_auth(cookies, |_| async move {
        let tick_btc_str =
            reqwest::get("https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD,RUB")
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
        let ticker_btc: api::TickerETH = (serde_json::from_str(&tick_btc_str)).unwrap();
        Ok(Json(ticker_btc))
    })
    .await
}

#[openapi(tag = "wallet")]
#[get("/userdata")]
pub async fn get_user_data(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Json<api::UserEth>> {
    require_auth(cookies, |cookie| async move {
        let user_id = cookie.value();
        {
            let state = state.lock().await;
            if let Some(user) = state.users.get(user_id) {
                let user_data_str =
                    reqwest::get("http://node.desolator.net/userdata/".to_owned() + &user.username)
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap();

                let user_data: api::UserEth = (serde_json::from_str(&user_data_str)).unwrap();
                Ok(Json(user_data))
            } else {
                Err(error::Error::NoUserFound.into())
            }
        }
    })
    .await
}

#[openapi(tag = "wallet")]
#[get("/erc20ticker/<token>")]
pub async fn erc20_ticker(
    cookies: &CookieJar<'_>,
    token: &str,
) -> error::Result<Json<api::TickerETH>> {
    require_auth(cookies, |_| async move {
        let url_req = "https://min-api.cryptocompare.com/data/price?fsym=".to_owned()
            + token
            + "&tsyms=USD,RUB";
        let ticker_erc20_str = reqwest::get(url_req).await.unwrap().text().await.unwrap();
        let ticker_erc20: api::TickerETH = (serde_json::from_str(&ticker_erc20_str)).unwrap();
        Ok(Json(ticker_erc20))
    })
    .await
}

#[openapi(tag = "wallet")]
#[get("/ethfee")]
pub async fn ethfee(cookies: &CookieJar<'_>) -> error::Result<Json<api::EthGasPrice>> {
    require_auth(cookies, |_| async move {
        let resurl = "https://api.etherscan.io/api?module=gastracker&action=gasoracle&apikey=P8AXZC7V71IJA4XPMFEIIYX9S2S4D8U3T6";

        let fee_eth_res = reqwest::get(resurl)
                                            .await
                                            .unwrap()
                                            .text()
                                            .await
                                            .unwrap();

        let fee_eth : api::EthFeeResp = (serde_json::from_str(&fee_eth_res)).unwrap();
        Ok(Json(fee_eth.result))
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
) -> error::Result<Json<api::History>> {
    fn to_deposit_history_item(deposit: &Transaction) -> api::HistoryItem {
        match deposit {
            Transaction::Btc(btc_deposit) => api::HistoryItem::Deposit(api::DepositHistoryItem {
                currency: Currency::BTC,
                date: btc_deposit.timestamp,
                number_of_confirmations: btc_deposit.confirmations,
                value: btc_deposit.amount.abs() as u64,
            }),
            Transaction::Eth() => todo!("Eth deposit history mapping"),
        }
    }

    fn to_withdrawal_history_item(
        currency: &Currency,
        withdrawal: &WithdrawalRequest,
    ) -> api::HistoryItem {
        let withdrawal_status = withdrawal.status.clone().into();

        api::HistoryItem::Withdrawal(api::WithdrawalHistoryItem {
            currency: currency.to_owned(),
            date: withdrawal.created_at,
            status: withdrawal_status,
            value: withdrawal.amount,
        })
    }
    require_auth_user(cookies, state, |_, user| async move {
        let mut history = user
            .currencies
            .iter()
            .flat_map(|(currency, info)| {
                let deposits = info.unconfirmed_transactions();
                let deposit_history = deposits.map(to_deposit_history_item);
                let withdrawals: Vec<_> = info.withdrawal_requests.values().collect();
                let withdrawal_history = withdrawals
                    .iter()
                    .map(|withdrawal| to_withdrawal_history_item(currency, withdrawal));

                withdrawal_history
                    .chain(deposit_history)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        history.sort_by(|a, b| api::history_item_time(b).cmp(api::history_item_time(a)));

        let history_slice = history.iter().skip(skip).take(take).cloned().collect();

        Ok(Json(api::History {
            target_number_of_confirmations: REQUIRED_NUMBER_OF_CONFIRMATIONS,
            history_items: history_slice,
        }))
    })
    .await
}

#[openapi(tag = "history")]
#[get("/historyeth")]
pub async fn get_history_eth(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Json<Vec<api::Erc20HistUnitU>>> {
    require_auth_user(cookies, state, |_, user| async move {
        let user_data =
            reqwest::get("http://node.desolator.net/userdata/".to_owned() + &user.username)
                .await
                .unwrap()
                .json::<api::UserEth>()
                .await
                .unwrap();
        Ok(Json(user_data.data.historyEth))
    })
    .await
}

#[openapi(tag = "history")]
#[get("/historyerc20/<token>")]
pub async fn get_history_erc20(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    token: String,
) -> error::Result<Json<Vec<api::Erc20HistUnitU>>> {
    require_auth_user(cookies, state, |_, user| async move {
        let user_data =
            reqwest::get("http://node.desolator.net/userdata/".to_owned() + &user.username)
                .await
                .unwrap()
                .json::<api::UserEth>()
                .await
                .unwrap();
        let mut history = vec![];
        for his_token in &user_data.data.historyTokens {
            if his_token.token.ticker == token {
                history = his_token.history.clone();
            };
        }
        Ok(Json(history))
    })
    .await
}

#[openapi(tag = "history")]
#[get("/withdraweth/<addr>/<amount>")]
pub async fn withdraw_eth(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    addr: String,
    amount: String,
) -> error::Result<()> {
    require_auth_user(cookies, state, |_, _| async move {
        let send_url = "http://node.desolator.net/sendtx/".to_owned() + &addr + "/" + &amount;
        reqwest::get(send_url).await.unwrap().text().await.unwrap();
        Ok(())
    })
    .await
}

#[openapi(tag = "withdraw")]
#[post("/withdraw", data = "<withdraw_request>")]
pub async fn post_withdraw(
    cookies: &CookieJar<'_>,
    btc: &State<BtcClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    state: &State<Arc<Mutex<DbState>>>,
    withdraw_request: Json<api::UserWithdrawRequest>,
) -> error::Result<()> {
    require_auth_user(cookies, state, |_, user| async move {
        let btc_fee_per_byte = &btc
            .get_fees()
            .await
            .map_err(|_| error::Error::FailedGetFee(Currency::BTC))?
            .fee_rate;

        let btc_balance = &user
            .currencies
            .get(&Currency::BTC)
            .ok_or(error::Error::NoUserCurrency(Currency::BTC))?
            .finalized_balance();
        let max_btc_amount_to_spend = btc_balance - btc_fee_per_byte * BTC_BYTES_PER_TRANSACTION;
        if max_btc_amount_to_spend >= withdraw_request.amount {
            let withdrawal_request = WithdrawalRequestInfo {
                id: Uuid::new_v4(),
                user: user.username,
                address: withdraw_request.address.to_owned(),
                amount: withdraw_request.amount,
            };
            let state_update =
                StateUpdate::new(UpdateBody::CreateWithdrawalRequest(withdrawal_request));
            updater
                .send(state_update)
                .await
                .map_err(|_| error::Error::NoUserFound.into())
        } else {
            Err(error::Error::InsufficientFunds(Currency::BTC).into())
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

    let packed_address = CurrencyAddress::BTC(BtcAddress {
        addr: format!("{}", address),
    });

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

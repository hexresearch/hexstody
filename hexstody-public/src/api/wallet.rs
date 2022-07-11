use log::*;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use rocket_okapi::openapi;

use super::auth::require_auth;
use hexstody_api::domain::{BtcAddress, Currency, CurrencyAddress};
use hexstody_api::error;
use hexstody_api::types as api;
use hexstody_btc_client::client::{BtcClient, BTC_BYTES_PER_TRANSACTION};
use hexstody_db::state::State as DbState;
use hexstody_db::state::WithdrawalRequestStatus;
use hexstody_db::state::{Transaction, WithdrawalRequest, REQUIRED_NUMBER_OF_CONFIRMATIONS};
use hexstody_db::update::deposit::DepositAddress;
use hexstody_db::update::{StateUpdate, UpdateBody};
use reqwest;


use serde::{Deserialize, Serialize};
use std::i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserETH {
    pub id: i32,
    pub login: String,
    pub address: String,

}


#[derive(Debug, Serialize, Deserialize)]
pub struct BalResp {
    pub status: String,
    pub message: String,
    pub result: String
}

#[openapi(tag = "wallet")]
#[get("/balance")]
pub async fn get_balance(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Json<api::Balance>> {
    require_auth(cookies, |cookie| async move {
        let user_id = cookie.value();
        {
            let state = state.lock().await;
            if let Some(user) = state.users.get(user_id) {
                let userETHstr = reqwest::get(&("http://localhost:8000/user/".to_owned()+&user.username))
                                                                                            .await
                                                                                            .unwrap()
                                                                                            .text()
                                                                                            .await
                                                                                            .unwrap();
                let userETH : UserETH = (serde_json::from_str(&userETHstr)).unwrap();



                let balance = reqwest::get(&("http://localhost:8000/balance2/".to_owned()+&userETH.address))
                                                                                            .await
                                                                                            .unwrap()
                                                                                            .text()
                                                                                            .await
                                                                                            .unwrap();
                let bal : BalResp = (serde_json::from_str(&balance)).unwrap();
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("User: {:?}",user.username);
                println!("userETH = {:?}", userETH);
                println!("body = {:?}", bal.result);
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                let brf = bal.result.parse::<u64>().unwrap();


                let mut balances: Vec<api::BalanceItem> = user
                    .currencies
                    .iter()
                    .map(|(cur, info)| api::BalanceItem {
                        currency: cur.clone(),
                        value: info.balance(),
                    })
                    .collect();

                let ethindex = balances.iter().position(|r| r.currency == Currency::ETH ).unwrap();


                balances[ethindex] = api::BalanceItem{currency: Currency::ETH, value: brf};
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("User: {:?}",user.username);
                println!("userETH = {:?}", userETH);
                println!("body = {:?}", bal.result);
                println!("brf = {:?}", brf);
                println!("ethindex = {:?}", ethindex);
                println!("balances = {:?}", balances);
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                println!("==========BALANCES==DEBUG================");
                Ok(Json(api::Balance { balances }))
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
) -> error::Result<Json<api::DepositInfo>> {
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

#[openapi(tag = "wallet")]
#[post("/depositETH", data = "<currency>")]
pub async fn get_deposit_eth(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    btc: &State<BtcClient>,
    currency: Json<Currency>,
) -> error::Result<Json<api::DepositInfo>> {
    require_auth(cookies, |cookie| async move {
        let user_id = cookie.value();
        let userETHstr = reqwest::get(&("http://localhost:8000/user/".to_owned()+&user_id))
                                                                                    .await
                                                                                    .unwrap()
                                                                                    .text()
                                                                                    .await
                                                                                    .unwrap();
        let userETH : UserETH = (serde_json::from_str(&userETHstr)).unwrap();
        println!("==========DEPOSIT==DEBUG================");
        println!("==========DEPOSIT==DEBUG================");
        println!("==========DEPOSIT==DEBUG================");
        println!("==========DEPOSIT==DEBUG================");
        println!("==========DEPOSIT==DEBUG================");
        println!("User: {:?}",user_id);
        println!("userETHstr: {:?}",userETH.address);
        println!("==========DEPOSIT==DEBUG================");
        println!("==========DEPOSIT==DEBUG================");
        println!("==========DEPOSIT==DEBUG================");
        println!("==========DEPOSIT==DEBUG================");
        println!("==========DEPOSIT==DEBUG================");
        Ok(Json(api::DepositInfo {
            address: format!("{}", &userETH.address),
        }))
    })
    .await
}


#[openapi(tag = "wallet")]
#[post("/ethticker", data = "<currency>")]
pub async fn eth_ticker(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    btc: &State<BtcClient>,
    currency: Json<Currency>,
) -> error::Result<Json<api::TickerETH>> {
    require_auth(cookies, |cookie| async move {
        let tickETHstr = reqwest::get("https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=USD,RUB")
                                                                                    .await
                                                                                    .unwrap()
                                                                                    .text()
                                                                                    .await
                                                                                    .unwrap();
        let tETH : api::TickerETH = (serde_json::from_str(&tickETHstr)).unwrap();
        println!("==========TICKER==DEBUG================");
        println!("==========TICKER==DEBUG================");
        println!("==========TICKER==DEBUG================");
        println!("==========TICKER==DEBUG================");
        println!("==========TICKER==DEBUG================");
        println!("TickerETH: {:?}",tETH);
        println!("==========TICKER==DEBUG================");
        println!("==========TICKER==DEBUG================");
        println!("==========TICKER==DEBUG================");
        println!("==========TICKER==DEBUG================");
        println!("==========TICKER==DEBUG================");
        Ok(Json(tETH))
    })
    .await
}


#[openapi(tag = "wallet")]
#[get("/ethfee")]
pub async fn ethfee(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Json<api::EthGasPrice>> {
    require_auth(cookies, |cookie| async move {
        let resurl = ("https://api.etherscan.io/api?module=gastracker&action=gasoracle&apikey=P8AXZC7V71IJA4XPMFEIIYX9S2S4D8U3T6");

        let feeETHres = reqwest::get(resurl)
                                            .await
                                            .unwrap()
                                            .text()
                                            .await
                                            .unwrap();

        let feeETH : api::EthFeeResp = (serde_json::from_str(&feeETHres)).unwrap();
        println!("==========FEE==DEBUG================");
        println!("==========FEE==DEBUG================");
        println!("==========FEE==DEBUG================");
        println!("==========FEE==DEBUG================");
        println!("Fee ETH: {:?}",feeETH.result);
        println!("==========FEE==DEBUG================");
        println!("==========FEE==DEBUG================");
        println!("==========FEE==DEBUG================");
        println!("==========FEE==DEBUG================");
        println!("==========FEE==DEBUG================");
        Ok(Json(feeETH.result))
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
        let withdrawal_status = match &withdrawal.status {
            WithdrawalRequestStatus::InProgress(n) => {
                api::WithdrawalRequestStatus::InProgress { confirmations: *n }
            }
            WithdrawalRequestStatus::Confirmed => api::WithdrawalRequestStatus::Confirmed,
            WithdrawalRequestStatus::Rejected => api::WithdrawalRequestStatus::Rejected,
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
                let userETHstr = reqwest::get(&("http://localhost:8000/user/".to_owned()+&user.username))
                                                                                            .await
                                                                                            .unwrap()
                                                                                            .text()
                                                                                            .await
                                                                                            .unwrap();
                let userETH : UserETH = (serde_json::from_str(&userETHstr)).unwrap();


                let resurl = ("https://api.etherscan.io/api?module=account&action=txlist&address=".to_owned() +
                             &userETH.address +
                             "&startblock=0&endblock=99999999&page=1&offset=10&sort=desc&apikey=P8AXZC7V71IJA4XPMFEIIYX9S2S4D8U3T6");

                let userETHHistStr = reqwest::get(resurl)
                                                        .await
                                                        .unwrap()
                                                        .text()
                                                        .await
                                                        .unwrap();

                let ethHistList: api::EthHistResp = (serde_json::from_str(&userETHHistStr)).unwrap();
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("UserAddress: {:?}",&userETH.address);
                println!("UserHistStr: {:?}",ethHistList);
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");

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
            } else {
                Err(error::Error::NoUserFound.into())
            }
        }
    })
    .await
}


#[openapi(tag = "history")]
#[get("/historyeth")]
pub async fn get_history_eth(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Json<Vec<api::EthHistUnitU>>> {
    require_auth(cookies, |cookie| async move {
        let user_id = cookie.value();
        {
            let state = state.lock().await;

            if let Some(user) = state.users.get(user_id) {
                let userETHstr = reqwest::get(&("http://localhost:8000/user/".to_owned()+&user.username))
                                                                                            .await
                                                                                            .unwrap()
                                                                                            .text()
                                                                                            .await
                                                                                            .unwrap();
                let userETH : UserETH = (serde_json::from_str(&userETHstr)).unwrap();


                let resurl = ("https://api.etherscan.io/api?module=account&action=txlist&address=".to_owned() +
                             &userETH.address +
                             "&startblock=0&endblock=99999999&page=1&offset=20&sort=desc&apikey=P8AXZC7V71IJA4XPMFEIIYX9S2S4D8U3T6");

                let userETHHistStr = reqwest::get(resurl)
                                                        .await
                                                        .unwrap()
                                                        .text()
                                                        .await
                                                        .unwrap();

                let ethHistPred : api::EthHistResp = (serde_json::from_str(&userETHHistStr)).unwrap();
                let ethHistList : Vec<api::EthHistUnit> = ethHistPred.result;
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("userETHHistStr: {:?}",userETHHistStr);
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("UserAddress: {:?}",&userETH.address);
                println!("UserHistStr: {:?}",ethHistList);
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");
                println!("==========HISTORY==DEBUG================");

                let ethHistListU : Vec<api::EthHistUnitU> = ethHistList.iter()
                                                                       .map(|x| {
                                                                           return api::EthHistUnitU {
                                                                           blockNumber : x.blockNumber.clone(),
                                                                           timeStamp : x.timeStamp.clone(),
                                                                           hash : x.hash.clone(),
                                                                           from : x.from.clone(),
                                                                           to : x.to.clone(),
                                                                           value : x.value.clone(),
                                                                           gas : x.gas.clone(),
                                                                           gasPrice : x.gasPrice.clone(),
                                                                           contractAddress : x.contractAddress.clone(),
                                                                           confirmations : x.confirmations.clone(),
                                                                           addr : userETH.address.clone()
                                                                        };
                                                                        }
                                                                    ).collect();


                Ok(Json(ethHistListU))
            } else {
                Err(error::Error::NoUserFound.into())
            }
        }
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
    require_auth(cookies, |cookie| async move {
        let user_id = cookie.value();
        {
            let state = state.lock().await;

            if let Some(user) = state.users.get(user_id) {

//                let amountWithoutPoint = &amount[0..amount.len() - 1];
                let sendUrl = &("http://localhost:8000/sendtx/".to_owned()+&addr+"/"+&amount);

                let userETHstr = reqwest::get(sendUrl)
                                                    .await
                                                    .unwrap()
                                                    .text()
                                                    .await
                                                    .unwrap();

                println!("==========WIRTHDRAWETH==DEBUG================");
                println!("==========WIRTHDRAWETH==DEBUG================");
                println!("==========WIRTHDRAWETH==DEBUG================");
                println!("addr: {:?}",addr);
                println!("amount: {:?}",amount);
                println!("res: {:?}",userETHstr);
                println!("==========WIRTHDRAWETH==DEBUG================");
                println!("==========WIRTHDRAWETH==DEBUG================");
                println!("==========WIRTHDRAWETH==DEBUG================");
                println!("==========WIRTHDRAWETH==DEBUG================");

                Ok(())
            } else {
                Err(error::Error::NoUserFound.into())
            }
        }
    })
    .await
}

use hexstody_db::update::withdrawal::WithdrawalRequestInfo;

#[openapi(tag = "withdraw")]
#[post("/withdraw", data = "<withdraw_request>")]
pub async fn post_withdraw(
    cookies: &CookieJar<'_>,
    btc: &State<BtcClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    state: &State<Arc<Mutex<DbState>>>,
    withdraw_request: Json<api::UserWithdrawRequest>,
) -> error::Result<()> {
    require_auth(cookies, |cookie| async move {
        let user_id = cookie.value();
        {
            let state = state.lock().await;
            if let Some(user) = state.users.get(user_id) {
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
                let max_btc_amount_to_spend =
                    btc_balance - btc_fee_per_byte * BTC_BYTES_PER_TRANSACTION;
                if max_btc_amount_to_spend >= withdraw_request.amount {
                    let withdrawal_request = WithdrawalRequestInfo {
                        id: Uuid::new_v4(),
                        user: user_id.to_owned(),
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

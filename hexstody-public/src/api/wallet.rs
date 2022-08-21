use std::sync::Arc;

use super::auth::{require_auth, require_auth_user};
use hexstody_api::domain::{
    filter_tokens, BtcAddress, Currency, CurrencyAddress, CurrencyTxId, Erc20Token,
};
use hexstody_api::error;
use hexstody_api::types::{self as api, GetTokensResponse, TokenActionRequest, TokenInfo, BalanceItem};
use hexstody_btc_client::client::{BtcClient, BTC_BYTES_PER_TRANSACTION};
use hexstody_db::state::State as DbState;
use hexstody_db::state::{Transaction, WithdrawalRequest, REQUIRED_NUMBER_OF_CONFIRMATIONS};
use hexstody_db::update::deposit::DepositAddress;
use hexstody_db::update::withdrawal::WithdrawalRequestInfo;
use hexstody_db::update::{StateUpdate, UpdateBody};
use hexstody_eth_client::client::EthClient;
use log::*;
use reqwest;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use rocket_okapi::openapi;
use hexstody_db::update::misc::{TokenUpdate, TokenAction};
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

#[openapi(tag = "wallet")]
#[get("/balance")]
pub async fn get_balance(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    eth_client: &State<EthClient>,
) -> error::Result<Json<api::Balance>> {
    require_auth_user(cookies, state, |_, user| async move {
        let user_data_resp = eth_client.get_user_data(&user.username).await;
        if let Err(e) = user_data_resp {
            return Err(error::Error::FailedETHConnection(e.to_string()).into());
        };
        let user_data = user_data_resp.unwrap();
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
                    limit_info: info.limit_info.clone()
                }
            })
            .collect();
        Ok(Json(api::Balance { balances }))
    })
    .await
}

#[openapi(tag="wallet")]
#[post("/balance", data="<currency>")]
pub async fn get_balance_by_currency(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    eth_client: &State<EthClient>,
    currency: Json<Currency>
) -> error::Result<Json<api::BalanceItem>> {
    let cur = currency.into_inner();
    let currency = cur.clone();
    let nofound_err = Err(error::Error::NoUserCurrency(cur.clone()).into());
    let resp = require_auth_user(cookies, state, |_, user| async move {
        match user.currencies.get(&cur) {
            Some(info) => {
                let limit_info = info.limit_info.clone();
                if cur == Currency::BTC{
                    return Ok((info.balance(), limit_info))
                } else {
                    let user_data_resp = eth_client.get_user_data(&user.username).await;
                    if let Err(e) = user_data_resp {
                        return Err(error::Error::FailedETHConnection(e.to_string()).into());
                    };
                    let user_data = user_data_resp.unwrap();
                    match cur.clone() {
                        Currency::BTC => return nofound_err, // this should not happen
                        Currency::ETH => return Ok((user_data.data.balanceEth.parse().unwrap(), limit_info)),
                        Currency::ERC20(token) => {
                            for tok in user_data.data.balanceTokens{
                                if tok.tokenName == token.ticker{
                                    return Ok((tok.tokenBalance.parse::<u64>().unwrap(), limit_info))
                                }
                            }
                            return nofound_err;
                        },
                    }
                }
            },
            None => return nofound_err,
        }
    })
    .await;
    resp.map(|(value, limit_info)| Json(BalanceItem{currency, value, limit_info}))
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
    eth_client: &State<EthClient>,
) -> error::Result<Json<api::DepositInfo>> {
    require_auth_user(cookies, state, |_, user| async move {
        eth_client
            .get_user_data(&user.username)
            .await
            .map_err(|e| error::Error::FailedETHConnection(e.to_string()).into())
            .map(|user_data| {
                Json(api::DepositInfo {
                    address: format!("{}", &user_data.address),
                })
            })
    })
    .await
}

#[openapi(tag = "wallet")]
#[post("/ticker", data = "<currency>")]
pub async fn ticker(
    cookies: &CookieJar<'_>,
    currency: Json<&str>,
) -> error::Result<Json<api::TickerETH>> {
    require_auth(cookies, |_| async move {
        let url = format!(
            "https://min-api.cryptocompare.com/data/price?fsym={}&tsyms=USD,RUB",
            currency.0
        );
        let tick_btc_str = reqwest::get(url).await.unwrap().text().await.unwrap();
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
    eth_client: &State<EthClient>,
) -> error::Result<Json<api::UserEth>> {
    require_auth_user(cookies, state, |_, user| async move {
        eth_client
            .get_user_data(&user.username)
            .await
            .map_err(|e| error::Error::FailedETHConnection(e.to_string()).into())
            .map(|user_data| Json(user_data))
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

#[openapi(tag = "wallet")]
#[get("/btcfee")]
pub async fn btcfee(cookies: &CookieJar<'_>, btc: &State<BtcClient>) -> error::Result<Json<u64>> {
    require_auth(cookies, |_| async move {
        let btc_fee_per_byte = &btc
            .get_fees()
            .await
            .map_err(|_| error::Error::FailedGetFee(Currency::BTC))?
            .fee_rate;
        Ok(Json(btc_fee_per_byte * BTC_BYTES_PER_TRANSACTION))
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
                to_address: CurrencyAddress::from(btc_deposit.address.clone()),
                txid: CurrencyTxId::from(btc_deposit.txid),
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
#[get("/withdraweth/<addr>/<amount>")]
pub async fn withdraw_eth(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    eth_client: &State<EthClient>,
    addr: String,
    amount: String,
) -> error::Result<()> {
    require_auth_user(cookies, state, |_, _| async move {
        eth_client
            .send_tx(&addr, &amount)
            .await
            .map_err(|e| error::Error::FailedETHConnection(e.to_string()).into())
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
        if let CurrencyAddress::ETH(eth_withdraw) = &withdraw_request.address {
            let send_url = "http://node.desolator.net/sendtx/".to_owned()
                + &eth_withdraw.to_string()
                + "/"
                + &withdraw_request.amount.to_string();
            reqwest::get(send_url).await.unwrap().text().await.unwrap();
            Ok(())
        } else {
            let btc_balance = user
                .currencies
                .get(&Currency::BTC)
                .ok_or(error::Error::NoUserCurrency(Currency::BTC))?
                .finalized_balance();
            let btc_fee_per_byte = &btc
                .get_fees()
                .await
                .map_err(|_| error::Error::FailedGetFee(Currency::BTC))?
                .fee_rate;
            let required_amount = withdraw_request.amount + btc_fee_per_byte * BTC_BYTES_PER_TRANSACTION;
            if  required_amount <= btc_balance {
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

#[openapi(tag = "profile")]
#[get("/profile/tokens/list")]
pub async fn list_tokens(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Json<GetTokensResponse>> {
    require_auth_user(cookies, state, |_, user| async move {
        let info = Currency::supported_tokens()
            .into_iter()
            .map(
                |token| match user.currencies.get(&Currency::ERC20(token.clone())) {
                    Some(c) => TokenInfo {
                        token: token.clone(),
                        balance: c.balance(),
                        finalized_balance: c.finalized_balance(),
                        is_active: true,
                    },
                    None => TokenInfo {
                        token: token.clone(),
                        balance: 0,
                        finalized_balance: 0,
                        is_active: false,
                    },
                },
            )
            .collect();
        Ok(Json(GetTokensResponse { tokens: info }))
    })
    .await
}

#[openapi(tag = "profile")]
#[post("/profile/tokens/enable", data = "<req>")]
pub async fn enable_token(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    eth_client: &State<EthClient>,
    req: Json<TokenActionRequest>,
) -> error::Result<()> {
    require_auth_user(cookies, state, |_, user| async move {
        let token = req.into_inner().token;
        let c = Currency::ERC20(token.clone());
        match user.currencies.get(&c) {
            Some(_) => Err(error::Error::TokenAlreadyEnabled(token).into()),
            None => {
                let state_update = StateUpdate::new(UpdateBody::UpdateTokens(TokenUpdate {
                    user: user.username.clone(),
                    token: token.clone(),
                    action: TokenAction::Enable,
                }));
                let upd = updater.send(state_update).await;
                match upd {
                    Ok(_) => {
                        let mut tokens = filter_tokens(user.currencies.keys().cloned().collect());
                        tokens.push(token);
                        eth_client
                            .post_tokens(&user.username, &tokens)
                            .await
                            .map_err(|e| error::Error::FailedETHConnection(e.to_string()).into())
                    }
                    Err(e) => Err(error::Error::TokenActionFailed(e.to_string()).into()),
                }
            }
        }
    })
    .await
}

#[openapi(tag = "profile")]
#[post("/profile/tokens/disable", data = "<req>")]
pub async fn disable_token(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    eth_client: &State<EthClient>,
    req: Json<TokenActionRequest>,
) -> error::Result<()> {
    require_auth_user(cookies, state, |_, user| async move {
        let token = req.into_inner().token;
        let cur = Currency::ERC20(token.clone());
        match user.currencies.get(&cur) {
            None => Err(error::Error::TokenAlreadyDisabled(token).into()),
            Some(info) => {
                if info.balance() > 0 {
                    Err(error::Error::TokenNonZeroBalance(token).into())
                } else {
                    let state_update = StateUpdate::new(UpdateBody::UpdateTokens(TokenUpdate {
                        user: user.username.clone(),
                        token: token.clone(),
                        action: TokenAction::Disable,
                    }));
                    let upd = updater.send(state_update).await;
                    match upd {
                        Ok(_) => {
                            let tokens: Vec<Erc20Token> = user
                                .currencies
                                .keys()
                                .into_iter()
                                .filter_map(|c| match c {
                                    Currency::ERC20(tok) => {
                                        if tok.ticker == token.ticker {
                                            None
                                        } else {
                                            Some(token.clone())
                                        }
                                    }
                                    _ => None,
                                })
                                .collect();
                            eth_client
                                .post_tokens(&user.username, &tokens)
                                .await
                                .map_err(|e| {
                                    error::Error::FailedETHConnection(e.to_string()).into()
                                })
                        }
                        Err(e) => Err(error::Error::TokenActionFailed(e.to_string()).into()),
                    }
                }
            }
        }
    })
    .await
}

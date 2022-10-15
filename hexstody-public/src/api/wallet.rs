use std::str::FromStr;
use std::sync::Arc;

use super::auth::require_auth_user;
use chrono::prelude::*;
use hexstody_api::domain::{
    filter_tokens, BtcAddress, Currency, CurrencyAddress, CurrencyTxId, ETHTxid, Erc20, Erc20Token,
    EthAccount, Symbol, CurrencyUnit,
};
use hexstody_api::error;
use hexstody_api::types::{
    self as api, BalanceItem, Erc20HistUnitU, ExchangeFilter, ExchangeRequest, GetTokensResponse,
    TokenActionRequest, TokenInfo, WithdrawalFilter, EthFeeResp, UnitTickedAmount
};
use hexstody_btc_client::client::{BtcClient, BTC_BYTES_PER_TRANSACTION};
use hexstody_db::state::exchange::ExchangeOrderUpd;
use hexstody_db::state::{Network, State as DbState, WithdrawalRequestType};
use hexstody_db::state::{Transaction, WithdrawalRequest, CONFIRMATIONS_CONFIG};
use hexstody_db::update::deposit::DepositAddress;
use hexstody_db::update::misc::{TokenAction, TokenUpdate};
use hexstody_db::update::withdrawal::WithdrawalRequestInfo;
use hexstody_db::update::{StateUpdate, UpdateBody};
use hexstody_eth_client::client::EthClient;
use hexstody_runtime_db::RuntimeState;
use hexstody_ticker_provider::client::TickerClient;
use log::*;
use reqwest;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use rocket_okapi::openapi;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

#[openapi(tag = "wallet")]
#[get("/balance")]
pub async fn get_balance(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    rstate: &State<Arc<Mutex<RuntimeState>>>,
    eth_client: &State<EthClient>,
    ticker_client: &State<TickerClient>
) -> error::Result<Json<api::Balance>> {
    require_auth_user(cookies, state, |_, user| async move {
        let user_data_resp = eth_client.get_user_data(&user.username).await;
        if let Err(e) = user_data_resp {
            return Err(error::Error::FailedETHConnection(e.to_string()).into());
        };
        let user_data = user_data_resp.unwrap();
        let mut rstate = rstate.lock().await;
        let mut balances: Vec<api::BalanceItem>= vec![];
        for (cur, info) in user.currencies.iter() {
            let ticker = rstate.symbol_to_symbols_generic(ticker_client, cur.symbol(), vec![Symbol::USD, Symbol::RUB]).await.ok();
            let mut bal = info.balance();
            match cur {
                Currency::BTC => {}
                Currency::ETH => {
                    bal = user_data.data.balanceEth.parse::<u64>().unwrap();
                }
                Currency::ERC20(token) => {
                    for tok in &user_data.data.balanceTokens {
                        if tok.tokenName == token.ticker {
                            bal = tok.tokenBalance.parse::<u64>().unwrap_or(0);
                        }
                    }
                }
            }
            let bal = api::BalanceItem {
                currency: cur.clone(),
                value: (bal, &info.unit).into(),
                limit_info: info.limit_info.clone(),
                ticker,
            };
            balances.push(bal);
        }
        balances.sort();
        // balances.sort_by(|b1, b2| b1.currency.cmp(&b2.currency));
        Ok(Json(api::Balance { balances }))
    })
    .await
}

#[openapi(tag = "wallet")]
#[post("/balance", data = "<currency>")]
pub async fn get_balance_by_currency(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    rstate: &State<Arc<Mutex<RuntimeState>>>,
    eth_client: &State<EthClient>,
    ticker_client: &State<TickerClient>,
    currency: Json<Currency>,
) -> error::Result<Json<api::BalanceItem>> {
    let cur = currency.into_inner();
    let currency = cur.clone();
    let nofound_err = Err(error::Error::NoUserCurrency(cur.clone()).into());
    let resp = require_auth_user(cookies, state, |_, user| async move {
        match user.currencies.get(&cur) {
            Some(info) => {
                let limit_info = info.limit_info.clone();
                let unit = info.unit.clone();
                if cur == Currency::BTC {
                    return Ok(((info.balance(), &unit).into(), limit_info));
                } else {
                    let user_data_resp = eth_client.get_user_data(&user.username).await;
                    if let Err(e) = user_data_resp {
                        return Err(error::Error::FailedETHConnection(e.to_string()).into());
                    };
                    let user_data = user_data_resp.unwrap();
                    match cur.clone() {
                        Currency::BTC => return nofound_err, // this should not happen
                        Currency::ETH => {
                            return Ok((
                                (user_data.data.balanceEth.parse().unwrap(), &unit).into(),
                                limit_info
                            ))
                        }
                        Currency::ERC20(token) => {
                            for tok in user_data.data.balanceTokens {
                                if tok.tokenName == token.ticker {
                                    return Ok((
                                        (user_data.data.balanceEth.parse().unwrap(), &unit).into(),
                                        limit_info,
                                    ));
                                }
                            }
                            return nofound_err;
                        }
                    }
                }
            }
            None => return nofound_err,
        }
    })
    .await;
    let mut rstate = rstate.lock().await;
    let ticker = rstate.symbol_to_symbols_generic(ticker_client, currency.symbol(), vec![Symbol::USD, Symbol::RUB]).await.ok();
    resp.map(|(value, limit_info)| {
        Json(BalanceItem {
            currency,
            value,
            limit_info,
            ticker
        })
    })
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


/// Get eth fee from external provider
pub async fn get_eth_fee() -> reqwest::Result<api::EthGasPrice>{
    let req_url = "https://api.etherscan.io/api?module=gastracker&action=gasoracle&apikey=P8AXZC7V71IJA4XPMFEIIYX9S2S4D8U3T6";
    let fee_eth_res : EthFeeResp = 
        reqwest::get(req_url)
            .await?
            .json()
            .await?;
    Ok(fee_eth_res.result)
}

#[openapi(tag = "wallet")]
#[post("/fee/get?<ticker>", data="<currency>")]
pub async fn get_fee(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    rstate: &State<Arc<Mutex<RuntimeState>>>,
    btc_client: &State<BtcClient>,
    ticker_client: &State<TickerClient>,
    currency: Json<Currency>,
    ticker: bool
) -> error::Result<Json<UnitTickedAmount>>{
    let currency = currency.into_inner();
    // symbol is used for fee ticker. For Eth and Erc20 we use Eth ticker
    let symbol = if matches!(currency, Currency::BTC) {Symbol::BTC} else {Symbol::ETH};
    let (fee, unit) = require_auth_user(cookies, state, |_, user| async move {
        if matches!(currency, Currency::BTC){
            let bytes_estimate = rstate.lock().await.fee_estimates.btc_bytes_per_tx;
            let btc_fee_per_kilobyte = &btc_client
                .get_fees()
                .await
                .map_err(|_| error::Error::FailedGetFee(currency))?
                .fee_rate;
            let fee = (btc_fee_per_kilobyte * bytes_estimate) / 1024;
            let unit = user.get_unit_by_currency(Currency::BTC);
            Ok((fee,unit))
        } else {
            let gas_limit = {
                let rstate = rstate.lock().await;
                if currency.is_token() {rstate.fee_estimates.erc20_tx_gas_limit} else {rstate.fee_estimates.eth_tx_gas_limit}
            };
            let gas_price = get_eth_fee()
                .await
                .map_err(|_| error::Error::FailedGetFee(currency))?
                .ProposeGasPrice.round() as u64;
            let fee = gas_limit * gas_price * 1_000_000_000; // 1_000_000_000 to convert gwei to wei
            let unit = user.get_unit_by_currency(Currency::ETH);
            Ok((fee,unit))
        }
    }).await?;

    let t = if ticker {
        rstate.lock().await.symbol_to_symbols_generic(ticker_client, symbol, vec![Symbol::USD, Symbol::RUB]).await.ok()
    } else {None};
    Ok(Json(UnitTickedAmount{ amount: fee, name: unit.name(), mul: unit.mul(), prec: unit.precision(), ticker: t }))
}

#[openapi(tag = "history")]
#[get("/history/<skip>/<take>?<filter>")]
pub async fn get_history(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    eth_client: &State<EthClient>,
    skip: usize,
    take: usize,
    filter: Option<WithdrawalFilter>,
) -> error::Result<Json<api::History>> {
    let filter = filter.unwrap_or(WithdrawalFilter::All);
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
            Transaction::Eth(_) => todo!("Eth deposit history mapping"),
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
            txid: None,
        })
    }

    fn to_eth_history(h: &Erc20HistUnitU) -> api::HistoryItem {
        let curr = Currency::from_str(&h.tokenName).unwrap();
        let time = Utc.timestamp(h.timeStamp.parse().unwrap(), 0);
        let val = h.value.parse().unwrap_or(u64::MAX); // MAX for strange entries with value bigger than u64
        if h.addr.to_uppercase() != h.from.to_ascii_uppercase() {
            api::HistoryItem::Deposit(api::DepositHistoryItem {
                currency: curr,
                date: time,
                number_of_confirmations: 0,
                value: val,
                to_address: CurrencyAddress::ETH(EthAccount {
                    account: h.addr.to_owned(),
                }),
                txid: CurrencyTxId::ETH(ETHTxid {
                    txid: h.hash.to_owned(),
                }),
            })
        } else {
            api::HistoryItem::Withdrawal(api::WithdrawalHistoryItem {
                currency: curr,
                date: time,
                status: api::WithdrawalRequestStatus::InProgress {
                    confirmations_minus_rejections: 0,
                },
                value: val,
                txid: Some(CurrencyTxId::ETH(ETHTxid {
                    txid: h.hash.to_owned(),
                })),
            })
        }
    }

    require_auth_user(cookies, state, |_, user| async move {
        let mut history = user
            .currencies
            .iter()
            .flat_map(|(currency, info)| {
                let deposits = info.unconfirmed_transactions();
                let deposit_history = deposits.map(to_deposit_history_item);
                let withdrawals: Vec<_> = info
                    .withdrawal_requests
                    .values()
                    .filter(|w| w.matches_filter(filter))
                    .collect();
                let withdrawal_history = withdrawals
                    .iter()
                    .map(|withdrawal| to_withdrawal_history_item(currency, withdrawal));

                withdrawal_history
                    .chain(deposit_history)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let user_data: Json<api::UserEth> = eth_client
            .get_user_data(&user.username)
            .await
            .map(|user_data| Json(user_data))
            .unwrap();
        let mut eth_and_tokens_history = user_data
            .data
            .historyTokens
            .iter()
            .flat_map(|h| h.history.iter())
            .chain(user_data.data.historyEth.iter())
            .map(|h| to_eth_history(h))
            .collect();
        history.append(&mut eth_and_tokens_history);
        history.sort_by(|a, b| api::history_item_time(b).cmp(api::history_item_time(a)));

        let history_slice = history.iter().skip(skip).take(take).cloned().collect();

        Ok(Json(api::History {
            confirmations_config: CONFIRMATIONS_CONFIG,
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
            .send_tx("testlogin", &addr, &amount)
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
        match &withdraw_request.address {
            CurrencyAddress::ETH(_) => {
                let withdrawal_request = WithdrawalRequestInfo {
                    id: Uuid::new_v4(),
                    user: user.username,
                    address: withdraw_request.address.to_owned(),
                    amount: withdraw_request.amount,
                    request_type: WithdrawalRequestType::OverLimit,
                };
                let state_update =
                    StateUpdate::new(UpdateBody::CreateWithdrawalRequest(withdrawal_request));
                info!("state_update: {:?}", state_update);
                updater
                    .send(state_update)
                    .await
                    .map_err(|_| error::Error::NoUserFound.into())
            }
            CurrencyAddress::ERC20(_) => {
                let withdrawal_request = WithdrawalRequestInfo {
                    id: Uuid::new_v4(),
                    user: user.username,
                    address: withdraw_request.address.to_owned(),
                    amount: withdraw_request.amount,
                    request_type: WithdrawalRequestType::OverLimit,
                };
                let state_update =
                    StateUpdate::new(UpdateBody::CreateWithdrawalRequest(withdrawal_request));
                info!("state_update: {:?}", state_update);
                updater
                    .send(state_update)
                    .await
                    .map_err(|_| error::Error::NoUserFound.into())
            }
            CurrencyAddress::BTC(_) => {
                let btc_cur = Currency::BTC;
                let btc_info = user
                    .currencies
                    .get(&btc_cur)
                    .ok_or(error::Error::NoUserCurrency(btc_cur.clone()))?;
                let btc_balance = btc_info.finalized_balance();
                let spent = btc_info.limit_info.spent;
                let limit = btc_info.limit_info.limit.amount;
                let btc_fee_per_kilobyte = &btc
                    .get_fees()
                    .await
                    .map_err(|_| error::Error::FailedGetFee(Currency::BTC))?
                    .fee_rate;
                let required_amount = withdraw_request.amount
                    + (btc_fee_per_kilobyte * BTC_BYTES_PER_TRANSACTION) / 1024;
                if required_amount <= btc_balance {
                    let req_type = if limit - spent >= required_amount {
                        WithdrawalRequestType::UnderLimit
                    } else {
                        WithdrawalRequestType::OverLimit
                    };
                    info!("req_type: {:?}", req_type);
                    let withdrawal_request = WithdrawalRequestInfo {
                        id: Uuid::new_v4(),
                        user: user.username,
                        address: withdraw_request.address.to_owned(),
                        amount: withdraw_request.amount,
                        request_type: req_type,
                    };
                    let state_update =
                        StateUpdate::new(UpdateBody::CreateWithdrawalRequest(withdrawal_request));
                    info!("state_update: {:?}", state_update);
                    updater
                        .send(state_update)
                        .await
                        .map_err(|_| error::Error::NoUserFound.into())
                } else {
                    Err(error::Error::InsufficientFunds(btc_cur))?
                }
            }
        }
    })
    .await
    .map_err(|e| e.into())
}

#[openapi(tag = "deposit")]
#[post("/deposit/address", data = "<currency>")]
pub async fn get_deposit_address_handle(
    btc_client: &State<BtcClient>,
    eth_client: &State<EthClient>,
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    currency: Json<Currency>,
) -> error::Result<Json<CurrencyAddress>> {
    require_auth_user(cookies, state, |_, user| async move {
        let currency = currency.into_inner();
        get_deposit_address(
            btc_client,
            eth_client,
            updater,
            state,
            &user.username,
            currency.clone(),
        )
        .await
        .map(|v| Json(v))
        .map_err(|_| error::Error::NoUserCurrency(currency).into())
    })
    .await
}

pub async fn get_deposit_address(
    btc_client: &State<BtcClient>,
    eth_client: &State<EthClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    state: &State<Arc<Mutex<DbState>>>,
    user_id: &str,
    currency: Currency,
) -> Result<CurrencyAddress, error::Error> {
    match currency {
        Currency::BTC => allocate_address(btc_client, eth_client, updater, user_id, currency).await,
        Currency::ETH | Currency::ERC20(_) => {
            let db_state = state.lock().await;
            let deposit_addresses: Vec<CurrencyAddress> = db_state
                .users
                .get(user_id)
                .ok_or(error::Error::NoUserFound)?
                .currencies
                .get(&currency)
                .ok_or(error::Error::NoUserCurrency(currency.clone()))?
                .deposit_info
                .clone();
            if deposit_addresses.is_empty() {
                allocate_address(btc_client, eth_client, updater, user_id, currency.clone()).await
            } else {
                Ok(deposit_addresses[0].clone())
            }
        }
    }
}

async fn allocate_address(
    btc_client: &State<BtcClient>,
    eth_client: &State<EthClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    user_id: &str,
    currency: Currency,
) -> Result<CurrencyAddress, error::Error> {
    match currency {
        Currency::BTC => allocate_btc_address(btc_client, updater, user_id).await,
        Currency::ETH => allocate_eth_address(eth_client, updater, user_id).await,
        Currency::ERC20(token) => allocate_erc20_address(eth_client, updater, user_id, token).await,
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

async fn allocate_eth_address(
    eth_client: &State<EthClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    user_id: &str,
) -> Result<CurrencyAddress, error::Error> {
    let addr = eth_client
        .allocate_address(&user_id)
        .await
        .map_err(|e| error::Error::FailedETHConnection(e.to_string()))?;
    let packed_address = CurrencyAddress::ETH(EthAccount { account: addr });
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

async fn allocate_erc20_address(
    eth_client: &State<EthClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    user_id: &str,
    token: Erc20Token,
) -> Result<CurrencyAddress, error::Error> {
    let addr = eth_client
        .allocate_address(&user_id)
        .await
        .map_err(|e| error::Error::FailedETHConnection(e.to_string()))?;
    let packed_address = CurrencyAddress::ERC20(Erc20 {
        token: token,
        account: EthAccount { account: addr },
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
        let mut info: Vec<TokenInfo> = Currency::supported_tokens()
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
        info.sort();
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

#[openapi(tag = "wallet")]
#[post("/exchange/order", data = "<req>")]
pub async fn order_exchange(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    ticker_client: &State<TickerClient>,
    rstate: &State<Arc<Mutex<RuntimeState>>>,
    req: Json<ExchangeRequest>,
) -> error::Result<()> {
    require_auth_user(cookies, state, |_, user| async move {
        let ExchangeRequest {
            currency_from,
            currency_to,
            amount_from,
        } = req.into_inner();
        let cinfo = user
            .currencies
            .get(&currency_from)
            .ok_or(error::Error::NoUserCurrency(currency_from.clone()))?;
        let balance = cinfo.balance();
        let mut rstate = rstate.lock().await;
        let from_symbol = currency_from.symbol();
        let to_symbol = currency_to.symbol();
        let rate = rstate
            .symbol_to_symbol_adjusted(ticker_client, from_symbol.to_owned(), to_symbol.to_owned())
            .await
            .map_err(|e| error::Error::GenericError(e.to_string()))?;
        let amount_to =
            (amount_from as f64 / from_symbol.exponent() * rate * to_symbol.exponent()) as u64;
        if balance < amount_from {
            return Err(error::Error::InsufficientFunds(currency_from).into());
        } else {
            let id = Uuid::new_v4();
            let created_at = chrono::offset::Utc::now().to_string();
            let req = ExchangeOrderUpd {
                user: user.username,
                currency_from,
                currency_to,
                amount_from,
                amount_to,
                id,
                created_at,
            };
            let upd = StateUpdate::new(UpdateBody::ExchangeRequest(req));
            updater
                .send(upd)
                .await
                .map_err(|e| error::Error::GenericError(e.to_string()).into())
        }
    })
    .await
}

#[openapi(tag = "wallet")]
#[get("/exchange/list?<filter>")]
pub async fn list_my_orders(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    filter: ExchangeFilter,
) -> error::Result<Json<Vec<hexstody_api::types::ExchangeOrder>>> {
    require_auth_user(cookies, state, |_, user| async move {
        let res = user.get_exchange_requests(filter);
        Ok(Json(res))
    })
    .await
}

#[openapi(tag = "wallet")]
#[get("/network")]
pub async fn get_network(network: &State<Network>) -> Json<Network> {
    Json(network.inner().clone())
}

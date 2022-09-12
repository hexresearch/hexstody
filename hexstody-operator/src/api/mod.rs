use figment::Figment;
use hexstody_sig::SignatureVerificationConfig;
use rocket::{
    fs::FileServer,
    serde::json::Json,
    State as RocketState,
    fairing::AdHoc,
    {get, post, routes, uri},
};
use rocket_dyn_templates::{context, Template};
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use std::{path::PathBuf, str, sync::Arc};
use tokio::sync::{mpsc, Mutex, Notify};
use uuid::Uuid;

use hexstody_api::{
    domain::Currency,
    error,
    types::{
        ConfirmationData, HotBalanceResponse, Invite, InviteRequest, InviteResp,
        LimitChangeDecisionType, LimitChangeOpResponse, LimitConfirmationData, SignatureData,
        WithdrawalRequest, WithdrawalRequestDecisionType,
    },
};
use hexstody_btc_client::client::BtcClient;
use hexstody_db::{
    state::{State as HexstodyState, REQUIRED_NUMBER_OF_CONFIRMATIONS},
    update::limit::LimitChangeData,
    update::{
        misc::InviteRec, StateUpdate,
        UpdateBody,
    },
    Pool,
};
use hexstody_eth_client::client::EthClient;

mod helpers;
use helpers::*;

#[openapi(skip)]
#[get("/")]
async fn index() -> Template {
    let context = context! {
        page_title: "Operator dashboard",
        parent: "base",
    };
    Template::render("index", context)
}

/// # Get all supported currencies
#[openapi(tag = "Currency")]
#[get("/currencies")]
async fn get_supported_currencies(
    signature_data: SignatureData,
    config: &RocketState<SignatureVerificationConfig>,
) -> error::Result<Json<Vec<Currency>>> {
    guard_op_signature_nomsg(
        &config,
        uri!(get_supported_currencies).to_string(),
        signature_data,
    )?;
    Ok(Json(Currency::supported()))
}

/// # Get required number of confirmations from operatros
#[openapi(tag = "Confirmations")]
#[get("/confirmations")]
async fn get_required_confrimations(
    signature_data: SignatureData,
    config: &RocketState<SignatureVerificationConfig>,
) -> error::Result<Json<i16>> {
    guard_op_signature_nomsg(
        &config,
        uri!(get_required_confrimations).to_string(),
        signature_data,
    )?;
    Ok(Json(REQUIRED_NUMBER_OF_CONFIRMATIONS))
}

/// # Hot wallet balance
#[openapi(tag = "Hot wallet balance")]
#[get("/hot-wallet-balance/<currency_name>")]
async fn get_hot_wallet_balance(
    signature_data: SignatureData,
    config: &RocketState<SignatureVerificationConfig>,
    btc_client: &RocketState<BtcClient>,
    eth_client: &RocketState<EthClient>,
    currency_name: &str,
) -> error::Result<Json<HotBalanceResponse>> {
    guard_op_signature_nomsg(
        &config,
        uri!(get_hot_wallet_balance(currency_name)).to_string(),
        signature_data,
    )?;
    let currency = Currency::get_by_name(currency_name)
        .ok_or(error::Error::UnknownCurrency(format!("{:?}", currency_name)))?;
    if currency == Currency::BTC {
        btc_client
            .get_hot_wallet_balance()
            .await
            .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
            .map(|v| Json(v))
    } else {
        eth_client
            .get_hot_wallet_balance(&currency)
            .await
            .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
            .map(|v| Json(v))
    }
}

/// # Get all withdrawal requests
#[openapi(tag = "Withdrawal request")]
#[get("/request/<currency_name>")]
async fn list(
    state: &RocketState<Arc<Mutex<HexstodyState>>>,
    signature_data: SignatureData,
    config: &RocketState<SignatureVerificationConfig>,
    currency_name: &str,
) -> error::Result<Json<Vec<WithdrawalRequest>>> {
    guard_op_signature_nomsg(
        &config,
        uri!(list(currency_name)).to_string(),
        signature_data,
    )?;
    let hexstody_state = state.lock().await;
    let withdrawal_requests = Vec::from_iter(
        hexstody_state
            .withdrawal_requests()
            .values()
            .cloned()
            .filter_map(|x| {
                if x.address.currency().ticker_lowercase() == currency_name {
                    Some(x.into())
                } else {
                    None
                }
            }),
    );
    Ok(Json(withdrawal_requests))
}

/// # Confirm withdrawal request
#[openapi(tag = "Withdrawal request")]
#[post("/confirm", format = "json", data = "<confirmation_data>")]
async fn confirm(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<ConfirmationData>,
    config: &RocketState<SignatureVerificationConfig>,
) -> error::Result<()> {
    let confirmation_data = confirmation_data.into_inner();
    guard_op_signature(
        &config,
        uri!(confirm).to_string(),
        signature_data,
        &confirmation_data,
    )?;
    let url = [config.domain.clone(), uri!(confirm).to_string()].join("");
    let state_update = StateUpdate::new(UpdateBody::WithdrawalRequestDecision(
        (
            confirmation_data,
            signature_data,
            WithdrawalRequestDecisionType::Confirm,
            url,
        )
            .into(),
    ));
    update_sender
        .send(state_update)
        .await
        .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
}

/// # Reject withdrawal request
#[openapi(tag = "Withdrawal request")]
#[post("/reject", format = "json", data = "<confirmation_data>")]
async fn reject(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<ConfirmationData>,
    config: &RocketState<SignatureVerificationConfig>,
) -> error::Result<()> {
    let confirmation_data = confirmation_data.into_inner();
    guard_op_signature(
        &config,
        uri!(reject).to_string(),
        signature_data,
        &confirmation_data,
    )?;
    let url = [config.domain.clone(), uri!(reject).to_string()].join("");
    let state_update = StateUpdate::new(UpdateBody::WithdrawalRequestDecision(
        (
            confirmation_data,
            signature_data,
            WithdrawalRequestDecisionType::Reject,
            url,
        )
            .into(),
    ));
    update_sender
        .send(state_update)
        .await
        .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
}

/// Generate an invite
#[openapi(tag = "Generate invite")]
#[post("/invite/generate", format = "json", data = "<req>")]
async fn gen_invite(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    state: &RocketState<Arc<Mutex<HexstodyState>>>,
    config: &RocketState<SignatureVerificationConfig>,
    signature_data: SignatureData,
    req: Json<InviteRequest>,
) -> error::Result<Json<InviteResp>> {
    let label = req.label.clone();
    guard_op_signature(
        &config,
        uri!(gen_invite).to_string(),
        signature_data,
        &req.into_inner(),
    )?;
    let invitor = signature_data.public_key.to_string();
    let mut invite = Invite {
        invite: Uuid::new_v4(),
    };
    {
        let hexstody_state = state.lock().await;
        while hexstody_state.invites.get(&invite).is_some() {
            invite = Invite {
                invite: Uuid::new_v4(),
            };
        }
    }
    let state_update = StateUpdate::new(UpdateBody::GenInvite(InviteRec {
        invite,
        invitor,
        label: label.clone(),
    }));
    update_sender
        .send(state_update)
        .await
        .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
        .map(|_| Json(InviteResp { invite, label }))
}

/// List operator's invites
#[openapi(tag = "List invites")]
#[get("/invite/listmy")]
async fn list_ops_invites(
    state: &RocketState<Arc<Mutex<HexstodyState>>>,
    config: &RocketState<SignatureVerificationConfig>,
    signature_data: SignatureData,
) -> error::Result<Json<Vec<InviteResp>>> {
    guard_op_signature_nomsg(&config, uri!(list_ops_invites).to_string(), signature_data)?;
    let invitor = signature_data.public_key.to_string();
    let hexstody_state = state.lock().await;
    let invites = hexstody_state
        .invites
        .values()
        .into_iter()
        .filter_map(|v| {
            if v.invitor == invitor {
                Some(InviteResp {
                    invite: v.invite.clone(),
                    label: v.label.clone(),
                })
            } else {
                None
            }
        })
        .collect();
    Ok(Json(invites))
}

#[openapi(skip)]
#[get("/changes")]
async fn get_all_changes(
    state: &RocketState<Arc<Mutex<HexstodyState>>>,
    config: &RocketState<SignatureVerificationConfig>,
    signature_data: SignatureData,
) -> error::Result<Json<Vec<LimitChangeOpResponse>>> {
    guard_op_signature_nomsg(&config, uri!(get_all_changes).to_string(), signature_data)?;
    let hexstody_state = state.lock().await;
    let changes = hexstody_state
        .users
        .values()
        .into_iter()
        .flat_map(|uinfo| {
            uinfo.limit_change_requests.values().map(|req| {
                let LimitChangeData {
                    id,
                    user,
                    created_at,
                    status,
                    currency,
                    limit: requested_limit,
                    ..
                } = req.clone();
                let current_limit = uinfo
                    .currencies
                    .get(&currency)
                    .unwrap()
                    .limit_info
                    .limit
                    .clone();
                LimitChangeOpResponse {
                    id,
                    user,
                    created_at,
                    currency,
                    current_limit,
                    requested_limit,
                    status,
                }
            })
        })
        .collect();
    Ok(Json(changes))
}

#[openapi(skip)]
#[post("/limits/confirm", data = "<confirmation_data>")]
async fn confirm_limits(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<LimitConfirmationData>,
    config: &RocketState<SignatureVerificationConfig>,
) -> error::Result<()> {
    let confirmation_data = confirmation_data.into_inner();
    guard_op_signature(
        &config,
        uri!(confirm_limits).to_string(),
        signature_data,
        &confirmation_data,
    )?;
    let url = [config.domain.clone(), uri!(confirm_limits).to_string()].join("");
    let state_update = StateUpdate::new(UpdateBody::LimitChangeDecision(
        (
            confirmation_data,
            signature_data,
            LimitChangeDecisionType::Confirm,
            url,
        )
            .into(),
    ));
    update_sender
        .send(state_update)
        .await
        .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
}

#[openapi(skip)]
#[post("/limits/reject", data = "<confirmation_data>")]
async fn reject_limits(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<LimitConfirmationData>,
    config: &RocketState<SignatureVerificationConfig>,
) -> error::Result<()> {
    let confirmation_data = confirmation_data.into_inner();
    guard_op_signature(
        &config,
        uri!(reject_limits).to_string(),
        signature_data,
        &confirmation_data,
    )?;
    let url = [config.domain.clone(), uri!(reject_limits).to_string()].join("");
    let state_update = StateUpdate::new(UpdateBody::LimitChangeDecision(
        (
            confirmation_data,
            signature_data,
            LimitChangeDecisionType::Reject,
            url,
        )
            .into(),
    ));
    update_sender
        .send(state_update)
        .await
        .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
}

pub async fn serve_api(
    pool: Pool,
    state: Arc<Mutex<HexstodyState>>,
    _state_notify: Arc<Notify>,
    start_notify: Arc<Notify>,
    update_sender: mpsc::Sender<StateUpdate>,
    btc_client: BtcClient,
    eth_client: EthClient,
    api_config: Figment,
) -> Result<(), rocket::Error> {
    let on_ready = AdHoc::on_liftoff("API Start!", |_| {
        Box::pin(async move {
            start_notify.notify_one();
        })
    });
    let static_path: PathBuf = api_config.extract_inner("static_path").unwrap();
    let _ = rocket::custom(api_config)
        .mount("/", FileServer::from(static_path))
        .mount("/", routes![index])
        .mount(
            "/",
            openapi_get_routes![
                list,                           // GET:  /request/${currency.toLowerCase()} 
                confirm,                        // POST: /confirm', 
                reject,                         // POST: /reject',
                get_hot_wallet_balance,         // GET:  /hot-wallet-balance/${currency.toLowerCase()} 
                get_supported_currencies,       // GET:  /currencies
                get_required_confrimations,     // GET:  /confirmations 
                gen_invite,                     // POST: /invite/generate 
                list_ops_invites,               // GET:  /invite/listmy 
                get_all_changes,                // GET:  /changes 
                confirm_limits,                 // POST: /limits/confirm 
                reject_limits                   // POST: /limits/reject 
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
        .attach(AdHoc::config::<SignatureVerificationConfig>())
        .attach(Template::fairing())
        .attach(on_ready)
        .launch()
        .await?;
    Ok(())
}

use figment::Figment;
use rocket::{
    fs::FileServer,
    response::status,
    serde::json::Json,
    State as RocketState,
    {fairing::AdHoc, response::status::Created},
    {get, post, routes, uri},
};
use rocket_dyn_templates::{context, Template};
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use uuid::Uuid;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, Mutex, Notify};

use hexstody_api::types::{
    ConfirmationData, SignatureData, 
    WithdrawalRequest, WithdrawalRequestInfo, 
    WithdrawalRequestDecisionType, HotBalanceResponse, 
    Invite, InviteRequest, InviteResp, LimitChangeOpResponse, LimitConfirmationData, LimitChangeDecisionType
};
use hexstody_api::error;
use hexstody_btc_client::client::BtcClient;
use hexstody_db::{
    state::State as HexstodyState,
    update::{withdrawal::{
        WithdrawalRequestInfo as WithdrawalRequestInfoDb,
    }, misc::InviteRec, limit::LimitChangeData},
    update::{StateUpdate, UpdateBody},
    Pool,
};

mod helpers;
use helpers::*;

#[openapi(skip)]
#[get("/")]
async fn index(state: &RocketState<Arc<Mutex<HexstodyState>>>) -> Template {
    let hexstody_state = state.lock().await;
    let withdrawal_requests: Vec<WithdrawalRequest> = Vec::from_iter(
        hexstody_state
            .withdrawal_requests()
            .values()
            .cloned()
            .map(|x| x.into()),
    );
    let context = context! {
        title: "Operator dashboard".to_owned(),
        parent: "base".to_owned(),
        withdrawal_requests,
    };
    Template::render("index", context)
}

/// # hot wallet balance
#[openapi(tag = "Hot balance")]
#[post("/hotbalance")]
async fn get_hot_balance(
    signature_data: SignatureData,
    config: &RocketState<Config>,
    btc_client: &RocketState<BtcClient>,
) -> error::Result<Json<HotBalanceResponse>> {
    guard_op_signature_nomsg(&config, uri!(get_hot_balance).to_string(), signature_data)?;
    btc_client.get_hot_balance()
        .await
        .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
        .map(|v| Json(v))
}

/// # Get all withdrawal requests
#[openapi(tag = "Withdrawal request")]
#[get("/request")]
async fn list(
    state: &RocketState<Arc<Mutex<HexstodyState>>>,
    signature_data: SignatureData,
    config: &RocketState<Config>,
) -> error::Result<Json<Vec<WithdrawalRequest>>> {
    guard_op_signature_nomsg(&config, uri!(list).to_string(), signature_data)?;
    let hexstody_state = state.lock().await;
    let withdrawal_requests = Vec::from_iter(
        hexstody_state
            .withdrawal_requests()
            .values()
            .cloned()
            .map(|x| x.into()),
    );
    // let adr = CurrencyAddress::BTC(hexstody_api::domain::BtcAddress{addr: "bc1q8xsxrzmkvlv7d742uqcztfq6qywesrqfwvd5jl".to_string()});
    // let withdrawal_requests = vec![
    //     WithdrawalRequest{ 
    //         id: Uuid::new_v4(), 
    //         user: "todo!()".to_string(), 
    //         address: adr.clone(), 
    //         created_at: "now".to_string(), 
    //         amount: 12345678, 
    //         confirmation_status: WithdrawalRequestStatus::InProgress { confirmations: 1 }
    //     },
    //     WithdrawalRequest{ 
    //         id: Uuid::new_v4(), 
    //         user: "todo!()".to_string(), 
    //         address: adr.clone(), 
    //         created_at: "now".to_string(), 
    //         amount: 12345678, 
    //         confirmation_status: WithdrawalRequestStatus::InProgress { confirmations: 2 }
    //     }];
    Ok(Json(withdrawal_requests))
}

/// # Create new withdrawal request
#[openapi(tag = "Withdrawal request")]
#[post("/request", format = "json", data = "<withdrawal_request_info>")]
async fn create(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    withdrawal_request_info: Json<WithdrawalRequestInfo>,
    config: &RocketState<Config>,
) -> error::Result<Created<Json<WithdrawalRequest>>> {
    let withdrawal_request_info = withdrawal_request_info.into_inner();
    guard_op_signature(&config, uri!(create).to_string(), signature_data, &withdrawal_request_info)?;
    let info: WithdrawalRequestInfoDb = withdrawal_request_info.into();
    let state_update = StateUpdate::new(UpdateBody::CreateWithdrawalRequest(info));
    // TODO: check that state update was correctly processed
    update_sender
        .send(state_update)
        .await
        .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
        .map(|_| status::Created::new("/request"))
}

/// # Confirm withdrawal request
#[openapi(tag = "Withdrawal request")]
#[post("/confirm", format = "json", data = "<confirmation_data>")]
async fn confirm(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<ConfirmationData>,
    config: &RocketState<Config>,
) -> error::Result<()> {
    let confirmation_data = confirmation_data.into_inner();
    guard_op_signature(&config, uri!(confirm).to_string(), signature_data, &confirmation_data)?;
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
    config: &RocketState<Config>,
) -> error::Result<()> {
    let confirmation_data = confirmation_data.into_inner();
    guard_op_signature(&config, uri!(reject).to_string(), signature_data, &confirmation_data)?;
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
#[post("/invite/generate", format = "json", data="<req>")]
async fn gen_invite(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    state: &RocketState<Arc<Mutex<HexstodyState>>>,
    config: &RocketState<Config>,
    signature_data: SignatureData,
    req: Json<InviteRequest>
) -> error::Result<Json<InviteResp>> {
    let label = req.label.clone();
    guard_op_signature(&config, uri!(gen_invite).to_string(), signature_data, &req.into_inner())?;
    let invitor = signature_data.public_key.to_string();
    let mut invite = Invite{invite: Uuid::new_v4()};
    {
        let hexstody_state = state.lock().await;
        while hexstody_state.invites.get(&invite).is_some() {
            invite = Invite{invite: Uuid::new_v4()};
        }
    }
    let state_update = StateUpdate::new(UpdateBody::GenInvite(InviteRec{ invite, invitor, label: label.clone()}));
    update_sender
        .send(state_update)
        .await
         .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
        .map(|_| Json(InviteResp{invite, label}))
}

/// List operator's invites
#[openapi(tag = "List invites")]
#[get("/invite/listmy")]
async fn list_ops_invites(
    state: &RocketState<Arc<Mutex<HexstodyState>>>,
    config: &RocketState<Config>,
    signature_data: SignatureData,
) -> error::Result<Json<Vec<InviteResp>>>{
    guard_op_signature_nomsg(&config, uri!(list_ops_invites).to_string(), signature_data)?;
    let invitor = signature_data.public_key.to_string();
    let hexstody_state = state.lock().await;
    let invites = hexstody_state.invites.values().into_iter().filter_map(|v| {
        if v.invitor == invitor {
            Some(InviteResp{ invite: v.invite.clone(), label: v.label.clone()})
        } else {None}
    }).collect();
    Ok(Json(invites))
}

#[openapi(skip)]
#[get("/changes")]
async fn get_all_changes(
    state: &RocketState<Arc<Mutex<HexstodyState>>>,
    config: &RocketState<Config>,
    signature_data: SignatureData,
) -> error::Result<Json<Vec<LimitChangeOpResponse>>>{
    guard_op_signature_nomsg(&config, uri!(get_all_changes).to_string(), signature_data)?;
    let hexstody_state = state.lock().await;
    let changes = hexstody_state.users.values().into_iter()
        .flat_map(|uinfo| uinfo.limit_change_requests.values().map(|req| {
            let LimitChangeData{ id, user, created_at, status, currency, limit: requested_limit, .. } = req.clone();
            let current_limit = uinfo.currencies.get(&currency).unwrap().limit_info.limit.clone();
            LimitChangeOpResponse{ id, user, created_at, currency, current_limit, requested_limit, status}
        }
        )).collect();
    Ok(Json(changes))
}

#[openapi(skip)]
#[post("/limits/confirm", data="<confirmation_data>")]
async fn confirm_limits(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<LimitConfirmationData>,
    config: &RocketState<Config>,
) -> error::Result<()>{
    let confirmation_data = confirmation_data.into_inner();
    guard_op_signature(&config, uri!(confirm_limits).to_string(), signature_data, &confirmation_data)?;
    let url = [config.domain.clone(), uri!(confirm_limits).to_string()].join("");
    let state_update = StateUpdate::new(UpdateBody::LimitChangeDecision(
        ( confirmation_data, signature_data, LimitChangeDecisionType::Confirm, url).into(),
    ));
    update_sender
        .send(state_update)
        .await
        .map_err(|e| error::Error::InternalServerError(format!("{:?}", e)).into())
}

#[openapi(skip)]
#[post("/limits/reject", data="<confirmation_data>")]
async fn reject_limits(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<LimitConfirmationData>,
    config: &RocketState<Config>,
) -> error::Result<()>{
    let confirmation_data = confirmation_data.into_inner();
    guard_op_signature(&config, uri!(reject_limits).to_string(), signature_data, &confirmation_data)?;
    let url = [config.domain.clone(), uri!(reject_limits).to_string()].join("");
    let state_update = StateUpdate::new(UpdateBody::LimitChangeDecision(
        ( confirmation_data, signature_data, LimitChangeDecisionType::Reject, url).into(),
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
        .mount("/", openapi_get_routes![
            list, 
            create, 
            confirm, 
            reject, 
            get_hot_balance, 
            gen_invite,
            list_ops_invites,
            get_all_changes,
            confirm_limits,
            reject_limits
            ])
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
        .attach(AdHoc::config::<Config>())
        .attach(Template::fairing())
        .attach(on_ready)
        .launch()
        .await?;
    Ok(())
}

use figment::Figment;
use p256::PublicKey;
use rocket::{
    fs::FileServer,
    http::Status,
    response::status,
    serde::{json, json::Json},
    State as RocketState,
    {fairing::AdHoc, response::status::Created},
    {get, post, routes, uri},
};
use rocket_dyn_templates::{context, Template};
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use serde::Deserialize;
use std::{path::PathBuf, str, sync::Arc};
use tokio::sync::{mpsc, Mutex, Notify};

use hexstody_api::types::{
    ConfirmationData, SignatureData, WithdrawalRequest, WithdrawalRequestInfo, WithdrawalRequestDecisionType
};
use hexstody_btc_client::client::BtcClient;
use hexstody_db::{
    state::State as HexstodyState,
    update::withdrawal::{
        WithdrawalRequestInfo as WithdrawalRequestInfoDb,
    },
    update::{StateUpdate, UpdateBody},
    Pool,
};
use hexstody_sig::SignatureVerificationData;

#[derive(Deserialize)]
struct Config {
    domain: String,
    operator_public_keys: Vec<PublicKey>,
}

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
        title: "Withdrawal requests".to_owned(),
        parent: "base".to_owned(),
        withdrawal_requests,
    };
    Template::render("index", context)
}

/// # Get all withdrawal requests
#[openapi(tag = "Withdrawal request")]
#[get("/request")]
async fn list(
    state: &RocketState<Arc<Mutex<HexstodyState>>>,
    signature_data: SignatureData,
    config: &RocketState<Config>,
) -> Result<Json<Vec<WithdrawalRequest>>, (Status, &'static str)> {
    let url = [config.domain.clone(), uri!(list).to_string()].join("");
    let signature_verification_data = SignatureVerificationData {
        url,
        signature: signature_data.signature,
        nonce: signature_data.nonce,
        message: None,
        public_key: signature_data.public_key,
    };
    let _ = signature_verification_data
        .verify(config.operator_public_keys.clone())
        .map_err(|_| (Status::Forbidden, "Signature verification failed"))?;
    let hexstody_state = state.lock().await;
    let withdrawal_requests = Vec::from_iter(
        hexstody_state
            .withdrawal_requests()
            .values()
            .cloned()
            .map(|x| x.into()),
    );
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
) -> Result<Created<Json<WithdrawalRequest>>, (Status, &'static str)> {
    let withdrawal_request_info = withdrawal_request_info.into_inner();
    let url = [config.domain.clone(), uri!(create).to_string()].join("");
    let message = json::to_string(&withdrawal_request_info).unwrap();
    let signature_verification_data = SignatureVerificationData {
        url,
        signature: signature_data.signature,
        nonce: signature_data.nonce,
        message: Some(message),
        public_key: signature_data.public_key,
    };
    let _ = signature_verification_data
        .verify(config.operator_public_keys.clone())
        .map_err(|_| (Status::Forbidden, "Signature verification failed"))?;
    let info: WithdrawalRequestInfoDb = withdrawal_request_info.into();
    let state_update = StateUpdate::new(UpdateBody::CreateWithdrawalRequest(info));
    // TODO: check that state update was correctly processed
    update_sender
        .send(state_update)
        .await
        .map_err(|_| (Status::InternalServerError, "Internal server error"))?;
    Ok(status::Created::new("/request"))
}

/// # Confirm withdrawal request
#[openapi(tag = "Withdrawal request")]
#[post("/confirm", format = "json", data = "<confirmation_data>")]
async fn confirm(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<ConfirmationData>,
    config: &RocketState<Config>,
) -> Result<(), (Status, &'static str)> {
    let confirmation_data = confirmation_data.into_inner();
    if confirmation_data.0.confirmation_status.is_some() {
        return Err((Status::BadRequest, "Confirmation status should be empty"));
    }
    let url = [config.domain.clone(), uri!(confirm).to_string()].join("");
    let message = json::to_string(&confirmation_data).unwrap();
    let signature_verification_data = SignatureVerificationData {
        url: url.clone(),
        signature: signature_data.signature,
        nonce: signature_data.nonce,
        message: Some(message.clone()),
        public_key: signature_data.public_key,
    };
    let _ = signature_verification_data
        .verify(config.operator_public_keys.clone())
        .map_err(|_| (Status::Forbidden, "Signature verification failed"))?;
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
        .map_err(|_| (Status::InternalServerError, "Internal server error"))?;
    Ok(())
}

/// # Reject withdrawal request
#[openapi(tag = "Withdrawal request")]
#[post("/reject", format = "json", data = "<confirmation_data>")]
async fn reject(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<ConfirmationData>,
    config: &RocketState<Config>,
) -> Result<(), (Status, &'static str)> {
    let confirmation_data = confirmation_data.into_inner();
    if confirmation_data.0.confirmation_status.is_some() {
        return Err((Status::BadRequest, "Confirmation status should be empty"));
    }
    let url = [config.domain.clone(), uri!(reject).to_string()].join("");
    let message = json::to_string(&confirmation_data).unwrap();
    let signature_verification_data = SignatureVerificationData {
        url: url.clone(),
        signature: signature_data.signature,
        nonce: signature_data.nonce,
        message: Some(message.clone()),
        public_key: signature_data.public_key,
    };
    let _ = signature_verification_data
        .verify(config.operator_public_keys.clone(),)
        .map_err(|_| (Status::Forbidden, "Signature verification failed"))?;
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
        .map_err(|_| (Status::InternalServerError, "Internal server error"))?;
    Ok(())
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
        .mount("/", openapi_get_routes![list, create, confirm, reject])
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

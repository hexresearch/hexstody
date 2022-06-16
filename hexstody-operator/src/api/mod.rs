use figment::Figment;
use p256::{
    ecdsa::{signature::Verifier, VerifyingKey},
    PublicKey,
};
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
    ConfirmationData, SignatureData, SignatureError, WithdrawalRequest, WithdrawalRequestInfo,
};
use hexstody_btc_client::client::BtcClient;
use hexstody_db::{
    state::State as HexstodyState,
    update::withdrawal::{
        WithdrawalRequestDecisionType, WithdrawalRequestInfo as WithdrawalRequestInfoDb,
    },
    update::{StateUpdate, UpdateBody},
    Pool,
};

#[derive(Deserialize)]
struct Config {
    domain: String,
    operator_public_keys: Vec<PublicKey>,
}

fn verify_signature(
    url: &str,
    msg: Option<&str>,
    signature_data: &SignatureData,
    operator_public_keys: &Vec<PublicKey>,
) -> Result<(), SignatureError> {
    if !operator_public_keys.contains(&signature_data.public_key) {
        return Err(SignatureError::UnknownPublicKey);
    };
    let msg_str: String = match msg {
        None => {
            let msg_items: [&str; 2] = [url, &signature_data.nonce.to_string()];
            msg_items.join(":")
        }
        Some(message) => {
            let msg_items: [&str; 3] = [url, &message, &signature_data.nonce.to_string()];
            msg_items.join(":")
        }
    };
    VerifyingKey::from(signature_data.public_key)
        .verify(msg_str.as_bytes(), &signature_data.signature)
        .map_err(|_| SignatureError::InvalidSignature)
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
    let _ = verify_signature(&url, None, &signature_data, &config.operator_public_keys)
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
    let msg = &json::to_string(&withdrawal_request_info).unwrap();
    let _ = verify_signature(
        &url,
        Some(&msg),
        &signature_data,
        &config.operator_public_keys,
    )
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
    let url = [config.domain.clone(), uri!(confirm).to_string()].join("");
    let msg = json::to_string(&confirmation_data).unwrap();
    let _ = verify_signature(
        &url,
        Some(&msg),
        &signature_data,
        &config.operator_public_keys,
    )
    .map_err(|_| (Status::Forbidden, "Signature verification failed"))?;
    let state_update = StateUpdate::new(UpdateBody::WithdrawalRequestDecision(
        (
            confirmation_data,
            signature_data,
            WithdrawalRequestDecisionType::Confirm,
            url,
            msg,
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
    let url = [config.domain.clone(), uri!(reject).to_string()].join("");
    let msg = json::to_string(&confirmation_data).unwrap();
    let _ = verify_signature(
        &url,
        Some(&msg),
        &signature_data,
        &config.operator_public_keys,
    )
    .map_err(|_| (Status::Forbidden, "Signature verification failed"))?;
    let state_update = StateUpdate::new(UpdateBody::WithdrawalRequestDecision(
        (
            confirmation_data,
            signature_data,
            WithdrawalRequestDecisionType::Reject,
            url,
            msg,
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

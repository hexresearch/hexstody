use base64;
use figment::Figment;
use log::*;
use okapi::openapi3::*;
use p256::ecdsa::{
    signature::{self, Verifier},
    Signature, VerifyingKey,
};
use p256::pkcs8::DecodePublicKey;
use rocket::fs::FileServer;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::status;
use rocket::serde::{json, json::json, json::Json, uuid::Uuid};
use rocket::State as RocketState;
use rocket::{fairing::AdHoc, response::status::Created};
use rocket::{get, post, routes, uri};
use rocket_dyn_templates::{context, Template};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Notify};

use hexstody_api::types::{WithdrawalRequest, WithdrawalRequestInfo};
use hexstody_btc_client::client::BtcClient;
use hexstody_db::state::{withdraw, State as HexstodyState};
use hexstody_db::update::{
    withdrawal::WithdrawalRequestInfo as WithdrawalRequestInfoDb, StateUpdate, UpdateBody,
};
use hexstody_db::Pool;

/// Signature data that comes from operators
/// when they sign or reject requests
#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureData {
    pub signature: Signature,
    pub nonce: u64,
    pub public_key: VerifyingKey,
}

#[derive(Debug)]
pub enum SignatureError {
    MissingSignatureData,
    InvalidSignatureDataLength,
    InvalidSignature,
    InvalidNonce,
    InvalidPublicKey,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SignatureData {
    type Error = SignatureError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Signature-Data") {
            None => {
                return Outcome::Failure((Status::BadRequest, SignatureError::MissingSignatureData))
            }
            Some(sig_data) => {
                let sig_data_vec: Vec<&str> = sig_data.split(':').collect();
                match sig_data_vec[..] {
                    [signature_str, nonce_str, public_key_str] => {
                        let signature = match base64::decode(signature_str) {
                            Ok(sig_der) => match Signature::from_der(&sig_der) {
                                Ok(sig) => sig,
                                Err(_) => {
                                    return Outcome::Failure((
                                        Status::BadRequest,
                                        SignatureError::InvalidSignature,
                                    ));
                                }
                            },
                            Err(_) => {
                                return Outcome::Failure((
                                    Status::BadRequest,
                                    SignatureError::InvalidSignature,
                                ));
                            }
                        };
                        let nonce = match nonce_str.parse::<u64>() {
                            Ok(n) => n,
                            Err(_) => {
                                return Outcome::Failure((
                                    Status::BadRequest,
                                    SignatureError::InvalidNonce,
                                ))
                            }
                        };
                        let public_key = match base64::decode(public_key_str) {
                            Ok(key_der) => match VerifyingKey::from_public_key_der(&key_der) {
                                Ok(key) => key,
                                Err(_) => {
                                    return Outcome::Failure((
                                        Status::BadRequest,
                                        SignatureError::InvalidPublicKey,
                                    ))
                                }
                            },
                            Err(_) => {
                                return Outcome::Failure((
                                    Status::BadRequest,
                                    SignatureError::InvalidPublicKey,
                                ))
                            }
                        };
                        return Outcome::Success(SignatureData {
                            signature: signature,
                            nonce: nonce,
                            public_key: public_key,
                        });
                    }
                    _ => {
                        return Outcome::Failure((
                            Status::BadRequest,
                            SignatureError::InvalidSignatureDataLength,
                        ))
                    }
                };
            }
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for SignatureData {
    fn from_request_input(
        gen: &mut OpenApiGenerator,
        _name: String,
        required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let schema = gen.json_schema::<String>();
        let description = Some(
            "Contains a string with a serialized digital signature,
            a nonce, and the corresponding public key.
            Format is: \"signature:nonce:public_key\".
            Where \"signature\" is in Base64 encoded DER format.
            \"nonce\" is an UTF-8 string containing 64-bit unsigned integer.
            \"public_key\" is in Base64 encoded DER format."
                .to_owned(),
        );
        let example = Some(json!("MEYCIQCIlvwe8VWpYMFR/0kEbIU+Wh8VU9V3NNxOxM6/obuY4gIhAMP9RzhIwIOekO2EAGONfn/jkERPXlM/U+k9q3uNyRTf:1654706913710:MDkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDIgADWlzihGEBq52xGU9C7rbuYs3hloPAmWPmCkf9XgqkBrY="));
        Ok(RequestHeaderInput::Parameter(Parameter {
            name: "Signature-Data".to_owned(),
            location: "header".to_owned(),
            description: description,
            required,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Schema {
                style: None,
                explode: None,
                allow_reserved: false,
                schema,
                example: example,
                examples: None,
            },
            extensions: Object::default(),
        }))
    }
}

fn verify_signature(
    url: &str,
    msg: Option<&str>,
    signature_data: &SignatureData,
) -> Result<(), signature::Error> {
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
    signature_data
        .public_key
        .verify(msg_str.as_bytes(), &signature_data.signature)
}

struct Domain(String);

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
    domain: &RocketState<Domain>,
) -> Result<Json<Vec<WithdrawalRequest>>, (Status, &'static str)> {
    let url = [domain.0.clone(), uri!(list).to_string()].join("");
    let _ = verify_signature(&url, None, &signature_data)
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
    domain: &RocketState<Domain>,
) -> Result<Created<Json<WithdrawalRequest>>, (Status, &'static str)> {
    let withdrawal_request_info = withdrawal_request_info.into_inner();
    let url = [domain.0.clone(), uri!(create).to_string()].join("");
    let msg = &json::to_string(&withdrawal_request_info).unwrap();
    let _ = verify_signature(&url, Some(&msg), &signature_data)
        .map_err(|_| (Status::Forbidden, "Signature verification failed"))?;
    let info: WithdrawalRequestInfoDb = withdrawal_request_info.into();
    let state_update = StateUpdate::new(UpdateBody::NewWithdrawalRequest(info));
    // TODO: check that state update was correctly processed
    update_sender
        .send(state_update)
        .await
        .map_err(|_| (Status::InternalServerError, "Internal server error"))?;
    Ok(status::Created::new("/request"))
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ConfirmationData {
    request_id: Uuid,
}

/// # Confirm withdrawal request
#[openapi(tag = "Withdrawal request")]
#[post("/confirm", format = "json", data = "<confirmation_data>")]
async fn confirm(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<ConfirmationData>,
    domain: &RocketState<Domain>,
) -> Result<(), (Status, &'static str)> {
    let confirmation_data = confirmation_data.into_inner();
    let url = [domain.0.clone(), uri!(confirm).to_string()].join("");
    let msg = json::to_string(&confirmation_data).unwrap();
    let _ = verify_signature(&url, Some(&msg), &signature_data)
        .map_err(|_| (Status::Forbidden, "Signature verification failed"))?;
    Ok(())
}

/// # Reject withdrawal request
#[openapi(tag = "Withdrawal request")]
#[post("/reject", format = "json", data = "<confirmation_data>")]
async fn reject(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    signature_data: SignatureData,
    confirmation_data: Json<ConfirmationData>,
    domain: &RocketState<Domain>,
) -> Result<(), (Status, &'static str)> {
    let confirmation_data = confirmation_data.into_inner();
    let url = [domain.0.clone(), uri!(reject).to_string()].join("");
    let msg = json::to_string(&confirmation_data).unwrap();
    let _ = verify_signature(&url, Some(&msg), &signature_data)
        .map_err(|_| (Status::Forbidden, "Signature verification failed"))?;
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
    let domain = Domain(api_config.extract_inner("domain").unwrap());
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
        .manage(domain)
        .attach(Template::fairing())
        .attach(on_ready)
        .launch()
        .await?;
    Ok(())
}

use rocket::fs::FileServer;
use rocket::http::Status;
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::State as RocketState;
use rocket::{get, post, routes};
use rocket_dyn_templates::Template;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Notify};

use hexstody_api::types::{WithdrawalRequest, WithdrawalRequestInfo};
use hexstody_db::state::State as HexstodyState;
use hexstody_db::update::{
    withdrawal::WithdrawalRequestInfo as WithdrawalRequestInfoDb, StateUpdate, UpdateBody,
};
use hexstody_db::Pool;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexHandlerContext {
    pub title: String,
    pub parent: String,
    pub withdrawal_requests: Vec<WithdrawalRequest>,
}

#[openapi(skip)]
#[get("/")]
async fn index(state: &RocketState<Arc<Mutex<HexstodyState>>>) -> Template {
    let hexstody_state = state.lock().await;
    let withdrawal_requests = Vec::from_iter(
        hexstody_state
            .withdrawal_requests()
            .values()
            .cloned()
            .map(|x| x.into()),
    );
    let context = IndexHandlerContext {
        title: "Withdrawal requests".to_owned(),
        parent: "operator/base".to_owned(),
        withdrawal_requests: withdrawal_requests,
    };
    Template::render("operator/index", context)
}

#[openapi(tag = "request")]
#[get("/request")]
async fn list(state: &RocketState<Arc<Mutex<HexstodyState>>>) -> Json<Vec<WithdrawalRequest>> {
    let hexstody_state = state.lock().await;
    let withdrawal_requests = Vec::from_iter(
        hexstody_state
            .withdrawal_requests()
            .values()
            .cloned()
            .map(|x| x.into()),
    );
    Json(withdrawal_requests)
}

#[openapi(tag = "request")]
#[post("/request", format = "json", data = "<withdrawal_request_info>")]
async fn create(
    update_sender: &RocketState<mpsc::Sender<StateUpdate>>,
    withdrawal_request_info: Json<WithdrawalRequestInfo>,
) -> Result<Created<Json<WithdrawalRequest>>, Status> {
    let info: WithdrawalRequestInfoDb = withdrawal_request_info.into_inner().into();
    let state_update = StateUpdate::new(UpdateBody::NewWithdrawalRequest(info));
    // TODO: check that state update was correctly processed
    update_sender
        .send(state_update)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Created::new("/request"))
}

pub async fn serve_operator_api(
    pool: Pool,
    state: Arc<Mutex<HexstodyState>>,
    _state_notify: Arc<Notify>,
    _start_notify: Arc<Notify>,
    port: u16,
    update_sender: mpsc::Sender<StateUpdate>,
    secret_key: Option<String>,
    static_path: String,
) -> Result<(), rocket::Error> {
    let zero_key =
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
    let secret_key = secret_key.unwrap_or_else(|| zero_key.to_owned());

    let figment = rocket::Config::figment()
        .merge(("secret_key", secret_key))
        .merge(("port", port));
    let _ = rocket::custom(figment)
        .mount("/", FileServer::from(static_path))
        .mount("/", routes![index])
        .mount("/", openapi_get_routes![list, create])
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
        .attach(Template::fairing())
        .launch()
        .await?;
    Ok(())
}

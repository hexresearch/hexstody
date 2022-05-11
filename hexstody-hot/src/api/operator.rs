use rocket::fs::{relative, FileServer};
use rocket::http::Status;
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::State as RocketState;
use rocket::{get, post, routes};
use rocket_dyn_templates::Template;
use rocket_okapi::{openapi, swagger_ui::*};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

use hexstody_db::queries::insert_update;
use hexstody_db::state::State as HexstodyState;
use hexstody_db::state::WithdrawalRequest;
use hexstody_db::update::{withdrawal::WithdrawalRequestInfo, StateUpdate, UpdateBody};
use hexstody_db::Pool;

#[openapi(skip)]
#[get("/")]
fn index() -> Template {
    let context = HashMap::from([("title", "Withdrawal requests"), ("parent", "base")]);
    Template::render("operator/index", context)
}

// #[openapi(tag = "request")]
#[get("/request")]
async fn list(state: &RocketState<Arc<Mutex<HexstodyState>>>) -> Json<Vec<WithdrawalRequest>> {
    let hexstody_state = state.lock().await;
    let withdrawal_requests = Vec::from_iter(hexstody_state.withdrawal_requests.values().cloned());
    Json(withdrawal_requests)
}

// #[openapi(tag = "request")]
#[post("/request", format = "json", data = "<withdrawal_request_info>")]
async fn create(
    pool: &RocketState<Pool>,
    withdrawal_request_info: Json<WithdrawalRequestInfo>,
) -> Result<Created<Json<WithdrawalRequest>>, Status> {
    let state_update = StateUpdate::new(UpdateBody::NewWithdrawalRequest(
        withdrawal_request_info.into_inner(),
    ));
    insert_update(&pool, state_update.body.clone(), Some(state_update.created))
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Created::new("/request"))
}

pub async fn serve_operator_api(
    pool: Pool,
    state: Arc<Mutex<HexstodyState>>,
    state_notify: Arc<Notify>,
    port: u16,
) -> Result<(), rocket::Error> {
    let figment = rocket::Config::figment().merge(("port", port));
    rocket::custom(figment)
        .mount("/", FileServer::from(relative!("static/")))
        .mount("/", routes![index, list, create])
        .mount(
            "/swagger/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .attach(Template::fairing())
        .manage(state)
        .manage(pool)
        .launch()
        .await?;
    Ok(())
}

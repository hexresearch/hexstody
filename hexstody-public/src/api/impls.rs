use std::sync::Arc;

use hexstody_api::error;
use hexstody_auth::types::ApiKey;
use hexstody_db::{state::State, update::StateUpdate};
use hexstody_invoices::types::{Invoice, CreateInvoiceReq};
use rocket::{http::CookieJar, get, State as RState, serde::json::Json, post, Route};
use rocket_okapi::{openapi, openapi_get_routes};
use tokio::sync::{Mutex, mpsc::Sender};
use uuid::Uuid;

pub fn invoices_api() -> Vec<Route> {
    openapi_get_routes![
        get_user_invoices,
        get_user_invoice,
        create_invoice,
    ]
}

#[openapi(tag = "invoice")]
#[get("/get")]
pub async fn get_user_invoices(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &RState<Arc<Mutex<State>>>,
) -> error::Result<Json<Vec<Invoice>>> {
    hexstody_invoices::routes::get_user_invoices(cookies, api_key, state).await
}

#[openapi(tag = "invoice")]
#[post("/get", data="<id>")]
pub async fn get_user_invoice(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &RState<Arc<Mutex<State>>>,
    id: Json<Uuid>
) -> error::Result<Json<Option<Invoice>>> {
    hexstody_invoices::routes::get_user_invoice(cookies, api_key, state, id).await
}

#[openapi(tag = "invoice")]
#[post("/create", data="<req>")]
pub async fn create_invoice(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &RState<Arc<Mutex<State>>>,
    updater: &RState<Sender<StateUpdate>>,
    req: Json<CreateInvoiceReq>
) -> error::Result<()> {
    hexstody_invoices::routes::create_invoice(cookies, api_key, state, updater, req).await
}
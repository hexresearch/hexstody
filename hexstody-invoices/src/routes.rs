use std::sync::Arc;

use hexstody_auth::{HasAuth, require_auth, types::ApiKey};
use rocket::{serde::json::Json, State, http::CookieJar};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{storage::InvoiceStorage, types::{Invoice, InvoiceStatus, CreateInvoiceReq}, error};

/// Macros have to be omitted, since they can't handle generics
/// #[openapi(tag = "invoice")]
/// #[get("/invoice/user/get")]
pub async fn get_user_invoices<S>(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<S>>>,
) -> error::Result<Json<Vec<Invoice>>> 
where 
    S: InvoiceStorage + HasAuth + Send 
{
    require_auth(cookies, api_key, state, |user_id| async move {
        let res = state.lock().await.get_user_invoices(&user_id).await;
        Ok(Json(res))
    }).await
} 

pub async fn get_user_invoice<S>(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<S>>>,
    id: Json<Uuid>
) -> error::Result<Json<Option<Invoice>>>
where
    S: InvoiceStorage + HasAuth + Send
{
    require_auth(cookies, api_key, state, |user_id| async move {
        Ok(Json(state.lock().await.get_user_invoice(&user_id, &id).await))
    }).await
}

pub async fn create_invoice<S>(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<S>>>,
    req: Json<CreateInvoiceReq>
) -> error::Result<()> 
where
    S: InvoiceStorage + HasAuth + Send 
{
    require_auth(cookies, api_key, state, |user_id| async move {
        let CreateInvoiceReq { currency, payment_method, amount, due, order_id, callback, contact_info, description } = req.into_inner();
        let mut state = state.lock().await;
        let address = state.allocate_invoice_address(&user_id, &currency).await?;
        let invoice =  Invoice {
            id:  Uuid::new_v4(),
            user: user_id,
            currency,
            payment_method,
            address,
            amount,
            created: chrono::offset::Utc::now(),
            due,
            order_id,
            contact_info,
            description,
            callback,
            status: InvoiceStatus::Created,
        };

        state.store_invoice(invoice).await
    }).await
}
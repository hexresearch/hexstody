use std::{future::Future, sync::Arc};

use async_trait::async_trait;
use rocket::{http::CookieJar, State};
use hexstody_api::error as h_error;

pub mod error;
pub mod types;

use error::Error;
use tokio::sync::{Mutex, MutexGuard};
use types::ApiKey;

pub const AUTH_COOKIE: &str = "user_id";

#[async_trait]
pub trait HasAuth {
    fn get_user_id_by_api_key(&self, api_key: ApiKey) -> Option<String>;
}

pub trait HasUserInfo<I> {
    fn get_user_info(&self, user_id: &str) -> Option<I>;
}

/// Helper for implementing endpoints that require authentication
pub async fn require_auth<F, Fut, S, R>(
    cookies: &CookieJar<'_>, 
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<S>>>,
    future: F
) -> h_error::Result<R>
where
    S: Send + HasAuth,
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = h_error::Result<R>>,
{
    if let Some(cookie) = cookies.get_private(AUTH_COOKIE) {
        let user_id = cookie.value().to_string();
        future(user_id).await
    } else {
        if let Some(api_key) = api_key {
            if let Some(user_id) = state.lock().await.get_user_id_by_api_key(api_key) {
                future(user_id).await
            } else {
                Err(Error::AuthRequired.into())
            }
        } else {
            Err(Error::AuthRequired.into())
        }
    }
}

/// More specific helper than 'require_auth' as it also locks state
/// for read only and fetches user info.
pub async fn require_auth_user<F, S, I, Fut, R>(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<S>>>,
    future: F,
) -> h_error::Result<R>
where
    S: Send + HasAuth + HasUserInfo<I>,
    F: FnOnce(MutexGuard<S>, I) -> Fut,
    Fut: Future<Output = h_error::Result<R>>,
{
    require_auth(cookies, api_key, state, |user_id| async move {
        {
            let state = state.lock().await;
            if let Some(user) = state.get_user_info(&user_id) {
                future(state, user).await
            } else {
                Err(error::Error::NoUserFound.into())
            }
        }
    })
    .await
}

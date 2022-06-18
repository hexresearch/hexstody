use hexstody_api::error;
use hexstody_api::types as api;
use hexstody_db::state::*;
use hexstody_db::update::*;
use hexstody_db::update::signup::*;
use pwhash::bcrypt;
use rocket_okapi::openapi;
use rocket::http::{Cookie, CookieJar};
use rocket::post;
use rocket::serde::json::Json;
use rocket::State as RState;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, MutexGuard};

#[openapi(tag = "auth")]
#[post("/signup/email", data = "<data>")]
pub async fn signup_email(
    state: &RState<Arc<Mutex<State>>>,
    updater: &RState<mpsc::Sender<StateUpdate>>,
    data: Json<api::SignupEmail>,
) -> error::Result<Json<()>> {
    if data.user.len() < error::MIN_USER_NAME_LEN {
        return Err(error::Error::UserNameTooShort.into());
    }
    if data.user.len() > error::MAX_USER_NAME_LEN {
        return Err(error::Error::UserNameTooLong.into());
    }
    if data.password.len() < error::MIN_USER_PASSWORD_LEN {
        return Err(error::Error::UserPasswordTooShort.into());
    }
    if data.password.len() > error::MAX_USER_PASSWORD_LEN {
        return Err(error::Error::UserPasswordTooLong.into());
    }

    {
        let mstate = state.lock().await;
        if let Some(_) = mstate.users.get(&data.user) {
            return Err(error::Error::SignupExistedUser.into());
        } else {
            let pass_hash = bcrypt::hash(&data.password).map_err(|e| error::Error::from(e))?;
            let upd = StateUpdate::new(UpdateBody::Signup(SignupInfo {
                username: data.user.clone(),
                auth: SignupAuth::Password(pass_hash),
            }));
            updater.send(upd).await.unwrap();
        }
    }
    Ok(Json(()))
}

const AUTH_COOKIE: &str = "user_id";

#[openapi(tag = "auth")]
#[post("/signin/email", data = "<data>")]
pub async fn signin_email(
    state: &RState<Arc<Mutex<State>>>,
    data: Json<api::SigninEmail>,
    cookies: &CookieJar<'_>,
) -> error::Result<Json<()>> {
    if data.user.len() < error::MIN_USER_NAME_LEN {
        return Err(error::Error::UserNameTooShort.into());
    }
    if data.user.len() > error::MAX_USER_NAME_LEN {
        return Err(error::Error::UserNameTooLong.into());
    }
    if data.password.len() < error::MIN_USER_PASSWORD_LEN {
        return Err(error::Error::UserPasswordTooShort.into());
    }
    if data.password.len() > error::MAX_USER_PASSWORD_LEN {
        return Err(error::Error::UserPasswordTooLong.into());
    }

    {
        let mstate = state.lock().await;
        if let Some(UserInfo {
            auth: SignupAuth::Password(pass_hash),
            ..
        }) = mstate.users.get(&data.user)
        {
            if bcrypt::verify(&data.password, pass_hash) {
                cookies.add_private(Cookie::new(AUTH_COOKIE, data.user.clone()));
                Ok(Json(()))
            } else {
                Err(error::Error::SigninFailed.into())
            }
        } else {
            Err(error::Error::SigninFailed.into())
        }
    }
}

#[openapi(tag = "auth")]
#[post("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> error::Result<Json<()>> {
    require_auth(cookies, |cookie| async move {
        cookies.remove(cookie);
        Ok(Json(()))
    })
    .await
}

/// Helper for implementing endpoints that require authentification
pub async fn require_auth<F, Fut, R>(cookies: &CookieJar<'_>, future: F) -> error::Result<R>
where
    F: FnOnce(Cookie<'static>) -> Fut,
    Fut: Future<Output = error::Result<R>>,
{
    if let Some(cookie) = cookies.get_private(AUTH_COOKIE) {
        future(cookie).await
    } else {
        Err(error::Error::AuthRequired.into())
    }
}

/// More specific helper than 'require_auth' as it alsow locks state 
/// fore read only and fetches user info.
pub async fn require_auth_user<F, Fut, R>(cookies: &CookieJar<'_>, state: &RState<Arc<Mutex<State>>>, future: F) -> error::Result<R>
where
    F: FnOnce(MutexGuard<State>, UserInfo) -> Fut,
    Fut: Future<Output = error::Result<R>>,
{
    require_auth(cookies, |cookie| async move {
        let user_id = cookie.value();
        {
            let state = state.lock().await;
            if let Some(user) = state.users.get(user_id).cloned() {
                future(state, user).await
            } else {
                Err(error::Error::NoUserFound.into())
            }
        }
    })
    .await
}

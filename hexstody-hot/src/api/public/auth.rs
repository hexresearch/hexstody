use hexstody_api::error;
use hexstody_api::types as api;
use hexstody_db::state::*;
use hexstody_db::update::signup::*;
use hexstody_db::update::*;
use pwhash::bcrypt;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::State as RState;
use rocket::{post};
use rocket_okapi::openapi;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

#[openapi(tag = "auth")]
#[post("/signup/email", data = "<data>")]
pub async fn signup_email(
    state: &RState<Arc<Mutex<State>>>,
    updater: &RState<mpsc::Sender<StateUpdate>>,
    data: Json<api::SignupEmail>,
) -> error::Result<()> {
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
) -> error::Result<()> {
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
pub async fn logout(
    cookies: &CookieJar<'_>,
) -> error::Result<()> {
    if let Some(cookie) = cookies.get_private(AUTH_COOKIE) {
        cookies.remove(cookie);
        Ok(Json(()))
    } else {
        Err(error::Error::AuthRequired.into())
    }
}
use hexstody_api::domain::Currency;
use hexstody_api::error;
use hexstody_api::types as api;
use hexstody_db::state::*;
use hexstody_db::update::signup::*;
use hexstody_db::update::*;
use hexstody_eth_client::client::EthClient;
use pwhash::bcrypt;
use rocket::get;
use rocket::http::{Cookie, CookieJar};
use rocket::post;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State as RState;
use rocket::uri;
use rocket_dyn_templates::Template;
use rocket_okapi::openapi;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, MutexGuard};

pub struct IsTestFlag(pub bool);

#[openapi(tag = "auth")]
#[post("/signup/email", data = "<data>")]
pub async fn signup_email(
    state: &RState<Arc<Mutex<State>>>,
    updater: &RState<mpsc::Sender<StateUpdate>>,
    eth_client:&RState<EthClient>,
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

    let (user_exists, invite_valid) = {
        let state = state.lock().await;
        let ue = state.users.contains_key(&data.user);
        let iv = state.invites.contains_key(&data.invite);
        (ue,iv)
    };
    if user_exists {
        return Err(error::Error::SignupExistedUser.into());
    } 
    if !invite_valid {
        return Err(error::Error::InviteNotFound.into())
    }
    else {
        // Create user
        if let Err(e) = eth_client.createuser(&data.user).await{
            return Err(error::Error::FailedETHConnection(e.to_string()).into())
        }

        // Set user's default tokens
        if let Err(e) = eth_client.post_tokens(&data.user, &Currency::default_tokens()).await{
            return Err(error::Error::FailedETHConnection(e.to_string()).into())
        }
        let pass_hash = bcrypt::hash(&data.password).map_err(|e| error::Error::from(e))?;
        let upd = StateUpdate::new(UpdateBody::Signup(SignupInfo {
            username: data.user.clone(),
            invite: data.invite.clone(),
            auth: SignupAuth::Password(pass_hash),
        }));
        updater.send(upd).await.unwrap();
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

/// Helper for implementing endpoints that require authentication
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

/// More specific helper than 'require_auth' as it also locks state
/// for read only and fetches user info.
pub async fn require_auth_user<F, Fut, R>(
    cookies: &CookieJar<'_>,
    state: &RState<Arc<Mutex<State>>>,
    future: F,
) -> error::Result<R>
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

#[openapi(skip)]
#[get("/signin")]
pub fn signin_page() -> Template {
    let context = HashMap::from([("title", "Sign In"), ("parent", "base")]);
    Template::render("signin", context)
}


#[openapi(skip)]
#[get("/removeuser/<user>")]
pub async fn remove_user(
    eth_client: &RState<EthClient>,
    state: &RState<Arc<Mutex<hexstody_db::state::State>>>,
    is_test: &RState<IsTestFlag>,
    user: &str
) -> Result<(), Redirect> {
    if is_test.0 {
        let _ = eth_client.remove_user(&user).await;
        let mut mstate = state.lock().await;
        mstate.users.remove(user);
        Ok(())
    } else {
        Err(Redirect::to(uri!(signin_page)))
    }

}

/// Redirect to signin page
pub fn goto_signin() -> Redirect{
    Redirect::to(uri!(signin_page))
}
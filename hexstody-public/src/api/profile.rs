use std::sync::Arc;

use hexstody_api::{types::{LimitApiResp, LimitChangeReq, LimitChangeResponse}, domain::{Currency, Language}};
use rocket::{get, http::CookieJar, State, serde::json::Json, response::Redirect, post};
use rocket_okapi::openapi;
use tokio::sync::{Mutex, mpsc};
use hexstody_db::{state::State as DbState, update::{StateUpdate, limit::{LimitChangeUpd, LimitCancelData}, UpdateBody, misc::SetLanguage}};
use hexstody_api::error;
use super::auth::{require_auth_user, goto_signin};

#[openapi(skip)]
#[get("/profile/limits/get")]
pub async fn get_user_limits(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<Json<Vec<LimitApiResp>>, Redirect>{
    require_auth_user(cookies, state, |_, user| async move {
        let infos = user.currencies.values().map(|cur_info| 
            LimitApiResp{ 
                limit_info: cur_info.limit_info.clone(), 
                currency: cur_info.currency.clone() 
            }).collect();
        Ok(Json(infos))
    }).await.map_err(|_| goto_signin())
}

#[openapi(skip)]
#[post("/profile/limits", data="<new_limits>")]
pub async fn request_new_limits(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    new_limits: Json<Vec<LimitChangeReq>>
) -> Result<error::Result<()>, Redirect> {
    let new_limits = new_limits.into_inner();
    let resp = require_auth_user(cookies, state, |_, user| async move {
        let filtered_limits : Vec<LimitChangeUpd> = new_limits.into_iter().filter_map(|l| {
            match user.currencies.get(&l.currency) {
                None => None,
                Some(ci) => if ci.limit_info.limit == l.limit{
                    None
                } else {
                    Some(LimitChangeUpd{
                        user: user.username.clone(),
                        currency: l.currency.clone(),
                        limit: l.limit.clone(),
                    })
                }
            }
        }).collect();
        if filtered_limits.is_empty(){
            Err(error::Error::InviteNotFound.into())
        } else {
           for req in filtered_limits {
            let state_update = StateUpdate::new(UpdateBody::LimitsChangeRequest(req));
            let _ = updater.send(state_update).await;
            }
            Ok(())
        }
    }).await;
    match resp {
        Ok(v) => Ok(Ok(v)),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => if err.1.code == 8 || err.1.code == 7 {
            Err(goto_signin())
        } else {
            Ok(Err(err))
        },
    }
}

#[openapi(skip)]
#[get("/profile/limits/changes")]
pub async fn get_user_limit_changes(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<Json<Vec<LimitChangeResponse>>, Redirect>{
    require_auth_user(cookies, state, |_, user| async move {
        let changes = user.limit_change_requests.values().map(|v| { v.clone().into() }).collect();
        Ok(Json(changes))
    }).await.map_err(|_| goto_signin())
}

#[openapi(skip)]
#[post("/profile/limits/cancel", data="<currency>")]
pub async fn cancel_user_change(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    currency: Json<Currency>
) -> Result<error::Result<()>, Redirect>{
    let resp = require_auth_user(cookies, state, |_, user| async move {
        match user.limit_change_requests.get(&currency){
            Some(v) => {
                let state_update = StateUpdate::new(UpdateBody::CancelLimitChange(
                    LimitCancelData{ id: v.id.clone(), user: user.username.clone(), currency: currency.into_inner().clone() }));
                let _ = updater.send(state_update).await;
                Ok(())
            },
            None => return Err(error::Error::LimChangeNotFound.into()),
        }
    }).await;
    match resp {
        Ok(v) => Ok(Ok(v)),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => if err.1.code == 8 || err.1.code == 7 {
            Err(goto_signin())
        } else {
            Ok(Err(err))
        },
    }
}

#[openapi(skip)]
#[post("/profile/language", data="<lang>")]
pub async fn set_language(
    cookies: &CookieJar<'_>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    lang: Json<Language>
) -> error::Result<()> {
    let lang = lang.into_inner();
    require_auth_user(cookies, state, |_, user| async move {
        if user.config.language == lang {
            Err(error::Error::LimitsNoChanges.into())
        } else {
            let _ = updater.send(StateUpdate::new(UpdateBody::SetLanguage(SetLanguage{ user: user.username, language: lang }))).await;
            Ok(())
        }
    }).await
}
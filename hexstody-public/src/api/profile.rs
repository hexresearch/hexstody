use std::{sync::Arc, fmt::Debug};
use base64;
use hexstody_api::{types::{LimitApiResp, LimitChangeReq, LimitChangeResponse, ConfigChangeRequest, LimitChangeFilter}, domain::{Currency, Language, Email, PhoneNumber, TgName, Unit, CurrencyUnit, UnitInfo, UserUnitInfo}};
use hexstody_auth::{types::ApiKey, require_auth_user};
use rocket::{get, http::CookieJar, State, serde::json::Json, response::Redirect, post};
use rocket_okapi::openapi;
use tokio::sync::{Mutex, mpsc};
use hexstody_db::{state::{State as DbState, UserConfig}, update::{StateUpdate, limit::{LimitChangeUpd, LimitCancelData}, UpdateBody, misc::{SetLanguage, ConfigUpdateData, SetPublicKey, SetUnit}}};
use hexstody_api::domain::error;
use p256::{pkcs8::DecodePublicKey, PublicKey};

use super::auth::goto_signin;

#[openapi(tag = "profile")]
#[get("/profile/limits/get")]
pub async fn get_user_limits(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
) -> Result<Json<Vec<LimitApiResp>>, Redirect>{
    require_auth_user(cookies, api_key, state, |_, user| async move {
        let mut infos: Vec<LimitApiResp> = user.currencies.values().map(|cur_info| 
            LimitApiResp{ 
                limit_info: cur_info.limit_info.clone(), 
                currency: cur_info.currency.clone() 
            }).collect();
        infos.sort();
        Ok(Json(infos))
    }).await.map_err(|_| goto_signin())
}

#[openapi(tag = "profile")]
#[post("/profile/limits", data="<new_limits>")]
pub async fn request_new_limits(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    new_limits: Json<Vec<LimitChangeReq>>
) -> Result<error::Result<()>, Redirect> {
    let new_limits = new_limits.into_inner();
    let resp = require_auth_user(cookies, api_key, state, |_, user| async move {
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
                let (upd, mut receiver) = StateUpdate::new_sync(UpdateBody::LimitsChangeRequest(req));
                let _ = updater.send(upd).await;
                if let Err(e) = receiver.recv().await.unwrap(){
                    return Err(e.into())
                }
            }
            Ok(())
        }
    }).await;
    match resp {
        Ok(v) => Ok(Ok(v)),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => if err.code == 8 || err.code == 7 {
            Err(goto_signin())
        } else {
            Ok(Err(err))
        },
    }
}

#[openapi(tag = "profile")]
#[get("/profile/limits/changes?<filter>")]
pub async fn get_user_limit_changes(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
    filter: Option<LimitChangeFilter>
) -> Result<Json<Vec<LimitChangeResponse>>, Redirect>{
    let filter = filter.unwrap_or(LimitChangeFilter::All);
    require_auth_user(cookies, api_key, state, |_, user| async move {
        let changes = user.limit_change_requests
            .values()
            .filter_map(|v| if v.matches_filter(filter) { Some(v.clone().into()) } else {None})
            .collect();
        Ok(Json(changes))
    }).await.map_err(|_| goto_signin())
}

#[openapi(tag = "profile")]
#[post("/profile/limits/cancel", data="<currency>")]
pub async fn cancel_user_change(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    currency: Json<Currency>
) -> Result<error::Result<()>, Redirect>{
    let resp = require_auth_user(cookies, api_key, state, |_, user| async move {
        match user.limit_change_requests.get(&currency){
            Some(v) => {
                let (upd, mut receiver) = StateUpdate::new_sync(UpdateBody::CancelLimitChange(
                    LimitCancelData{ id: v.id.clone(), user: user.username.clone(), currency: currency.into_inner().clone() }));
                let _ = updater.send(upd).await;
                if let Err(e) = receiver.recv().await.unwrap(){
                    return Err(e.into())
                }
                Ok(())
            },
            None => return Err(error::Error::LimChangeNotFound.into()),
        }
    }).await;
    match resp {
        Ok(v) => Ok(Ok(v)),
        // Error code 8 => NoUserFound (not logged in). 7 => Requires auth
        Err(err) => if err.code == 8 || err.code == 7 {
            Err(goto_signin())
        } else {
            Ok(Err(err))
        },
    }
}

#[openapi(tag = "profile")]
#[post("/profile/language", data="<lang>")]
pub async fn set_language(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    lang: Json<Language>
) -> error::Result<()> {
    let lang = lang.into_inner();
    require_auth_user(cookies, api_key, state, |_, user| async move {
        if user.config.language == lang {
            Err(error::Error::LimitsNoChanges.into())
        } else {
            let (upd, mut receiver) = StateUpdate::new_sync(UpdateBody::SetLanguage(SetLanguage{ user: user.username, language: lang }));
            let _ = updater.send(upd).await;
            if let Err(e) = receiver.recv().await.unwrap(){
                return Err(e.into())
            }
            Ok(())
        }
    }).await
}

#[openapi(tag = "profile")]
#[get("/profile/settings/config")]
pub async fn get_user_config(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Json<UserConfig>>{
    require_auth_user(cookies, api_key, state, |_, user| async move {
        Ok(Json(user.config))
    }).await
}
#[openapi(tag = "profile")]
#[post("/profile/settings/config", data="<request>")]
pub async fn set_user_config(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    request: Json<ConfigChangeRequest>
) -> error::Result<()> {
    require_auth_user(cookies, api_key, state, |_, user| async move {
        let req = request.into_inner();
        let mut upd_data = ConfigUpdateData::default();
        upd_data.user = user.username;
        if let Some(email_str) = req.email {
            if !email_str.is_empty(){
                match Email::from_str(email_str.as_str()){
                    Some(email) => upd_data.email = Some(Ok(email)),
                    None => return Err(error::Error::InvalidEmail.into()),
                }
            } else {
                upd_data.email = Some(Err(()))
            }
        }
        if let Some(phone_str) = req.phone {
            if !phone_str.is_empty(){
                match PhoneNumber::from_str(phone_str.as_str()){
                    Some(phone) => upd_data.phone = Some(Ok(phone)),
                    None => return Err(error::Error::InvalidPhoneNumber.into()),
                }
            } else {
                upd_data.phone = Some(Err(()))
            }
        }
        upd_data.tg_name = req.tg_name.map(|tg_name| if tg_name.is_empty() {Err(())} else {Ok(TgName{tg_name})});
        let (upd, mut receiver) = StateUpdate::new_sync(UpdateBody::ConfigUpdate(upd_data));
        let _ = updater.send(upd).await;
        receiver.recv().await.unwrap().map_err(|e| e.into()).map(|_| ())
    }).await
}

fn to_generic_error<T, E>(e: E) -> error::Result<T>
where
E: Debug
{
    Err(error::Error::GenericError(format!("{:?}", e)).into())
}

#[openapi(tag = "profile")]
#[post("/profile/key", data="<key_b64>")]
pub async fn set_user_public_key(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    key_b64: Option<Json<String>>
) -> error::Result<()> {
    require_auth_user(cookies, api_key, state, |_, user| async move {
    let mut upd = SetPublicKey { user: user.username, public_key: None };
    if let Some(key_bytes) = key_b64 {
        match base64::decode(key_bytes.into_inner()){
            Ok(key_der) => {
                match PublicKey::from_public_key_der(&key_der){
                    Ok(public_key) => {
                        upd.public_key = Some(public_key);
                    },
                    Err(e) => return to_generic_error(e),
                }
            },
            Err(e) => return to_generic_error(e),
        }
    };
    let (upd, mut receiver) = StateUpdate::new_sync(UpdateBody::SetPublicKey(upd));
    let _ = updater.send(upd).await;
    receiver.recv().await.unwrap().map_err(|e| e.into()).map(|_| ())  
    }).await
}

#[openapi(tag = "profile")]
#[post("/profile/unit/set", data="<unit_reqs>")]
pub async fn set_unit(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    unit_reqs: Json<Vec<Unit>>
) -> error::Result<()> {
    let unit_reqs = unit_reqs.into_inner();
    require_auth_user(cookies, api_key, state, |_, user| async move {
        for unit_req in unit_reqs {
            let cur = unit_req.currency().ok_or(error::Error::UnknownCurrency(unit_req.name()))?;
            let cinfo = user.currencies.get(&cur).ok_or(error::Error::NoUserCurrency(cur))?;
            if cinfo.unit != unit_req {
                let (upd, mut receiver) = StateUpdate::new_sync(UpdateBody::SetUnit(SetUnit{ user: user.username.clone(), unit: unit_req }));
                let _ = updater.send(upd).await;
                if let Err(e) = receiver.recv().await.unwrap(){
                    return Err(e.into())
                }
            };
        }
        Ok(())
    }).await
}

#[openapi(tag = "profile")]
#[post("/profile/unit/get", data="<currency>")]
pub async fn get_unit(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
    currency: Json<Currency>
) -> error::Result<Json<Unit>> {
    let currency = currency.into_inner();
    require_auth_user(cookies, api_key, state, |_, user| async move {
        let cinfo = user.currencies.get(&currency).ok_or(error::Error::NoUserCurrency(currency))?;
        Ok(Json(cinfo.unit.clone()))
    }).await
}

#[openapi(tag = "profile")]
#[get("/profile/unit/all")]
pub async fn get_all_units(
    cookies: &CookieJar<'_>,
    api_key: Option<ApiKey>,
    state: &State<Arc<Mutex<DbState>>>,
) -> error::Result<Json<Vec<UserUnitInfo>>> {
    require_auth_user(cookies, api_key, state, |_, user| async move {
        let units: Vec<UnitInfo> =  user.currencies.values()
            .filter_map(|cinfo|
                if cinfo.unit.is_generic() {
                    None
                } else {
                    Some(cinfo.unit.clone().into())
                }
            ).collect();
        let mut info: Vec<UserUnitInfo> = units
                .into_iter()
                .map(|ui| (ui.unit.currency().unwrap(), ui).into())
                .collect();
        info.sort_by(|a,b| a.currency.cmp(&b.currency));
        Ok(Json(info))
    }).await
}
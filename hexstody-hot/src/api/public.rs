use chrono::prelude::*;
use hexstody_api::domain::currency::Currency;
use hexstody_api::error;
use hexstody_api::types as api;
use hexstody_db::state::*;
use hexstody_db::update::signup::*;
use hexstody_db::update::*;
use hexstody_db::Pool;
use pwhash::bcrypt;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::response::content;
use rocket::serde::json::Json;
use rocket::State as RState;
use rocket::{get, post, routes};
use rocket_dyn_templates::Template;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, Notify};

#[openapi(tag = "ping")]
#[get("/ping")]
fn ping() -> content::Json<()> {
    content::Json(())
}

#[openapi(tag = "get_balance")]
#[get("/get_balance")]
fn get_balance() -> Json<api::Balance> {
    let x = api::Balance {
        balances: vec![
            api::BalanceItem {
                currency: Currency::BTC,
                value: u64::MAX,
            },
            api::BalanceItem {
                currency: Currency::ETH,
                value: u64::MAX,
            },
        ],
    };

    Json(x)
}

#[openapi(tag = "get_history")]
#[get("/get_history/<skip>/<take>")]
fn get_history(skip: u32, take: u32) -> Json<History> {
    let x = History {
        target_number_of_confirmations: 6,
        history_items: vec![
            api::HistoryItem::Deposit(api::DepositHistoryItem {
                currency: Currency::BTC,
                date: Utc::now().naive_utc(),
                value: u64::MAX,
                number_of_confirmations: 3,
            }),
            api::HistoryItem::Withdrawal(api::WithdrawalHistoryItem {
                currency: Currency::ETH,
                date: Utc::now().naive_utc(),
                value: u64::MAX,
                status: api::WithdrawalRequestStatus::InProgress,
            }),
        ],
    };

    Json(x)
}

#[openapi(skip)]
#[get("/")]
fn index() -> Template {
    let context = HashMap::from([("title", "Index"), ("parent", "base")]);
    Template::render("index", context)
}

#[openapi(skip)]
#[get("/overview")]
fn overview() -> Template {
    let context = HashMap::from([("title", "Overview"), ("parent", "base")]);
    Template::render("overview", context)
}

#[openapi(tag = "auth")]
#[post("/signup/email", data = "<data>")]
async fn signup_email(
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

#[openapi(tag = "auth")]
#[post("/signin/email", data = "<data>")]
async fn signin_email(
    state: &RState<Arc<Mutex<State>>>,
    data: Json<api::SigninEmail>,
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
                Ok(Json(()))
            } else {
                Err(error::Error::SigninFailed.into())
            }
        } else {
            Err(error::Error::SigninFailed.into())
        }
    }
}

pub async fn serve_public_api(
    pool: Pool,
    state: Arc<Mutex<State>>,
    state_notify: Arc<Notify>,
    start_notify: Arc<Notify>,
    port: u16,
    update_sender: mpsc::Sender<StateUpdate>,
) -> Result<(), rocket::Error> {
    let figment = rocket::Config::figment().merge(("port", port));
    let on_ready = AdHoc::on_liftoff("API Start!", |_| {
        Box::pin(async move {
            start_notify.notify_one();
        })
    });

    rocket::custom(figment)
        .mount("/static", FileServer::from(relative!("static/")))
        .mount(
            "/",
            openapi_get_routes![ping, get_balance, get_history, signup_email, signin_email],
        )
        .mount("/", routes![index, overview])
        .mount(
            "/swagger/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .manage(state)
        .manage(update_sender)
        .attach(Template::fairing())
        .attach(on_ready)
        .launch()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;
    use futures::FutureExt;
    use futures_util::future::TryFutureExt;
    use hexstody_client::client::HexstodyClient;
    use std::panic::AssertUnwindSafe;

    const SERVICE_TEST_PORT: u16 = 8000;
    const SERVICE_TEST_HOST: &str = "127.0.0.1";

    async fn run_api_test<F, Fut>(pool: Pool, test_body: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        let _ = env_logger::builder().is_test(true).try_init();

        let state_mx = Arc::new(Mutex::new(State::default()));
        let state_notify = Arc::new(Notify::new());
        let start_notify = Arc::new(Notify::new());

        let (sender, receiver) = tokio::sync::oneshot::channel();
        let (update_sender, _) = tokio::sync::mpsc::channel(1000);
        tokio::spawn({
            let state = state_mx.clone();
            let state_notify = state_notify.clone();
            let start_notify = start_notify.clone();
            async move {
                let serve_task = serve_public_api(
                    pool,
                    state,
                    state_notify,
                    start_notify,
                    SERVICE_TEST_PORT,
                    update_sender,
                );
                futures::pin_mut!(serve_task);
                futures::future::select(serve_task, receiver.map_err(drop)).await;
            }
        });
        start_notify.notified().await;

        let res = AssertUnwindSafe(test_body()).catch_unwind().await;

        sender.send(()).unwrap();

        assert!(res.is_ok());
    }

    #[sqlx_database_tester::test(pool(
        variable = "pool",
        migrations = "../hexstody-db/migrations"
    ))]
    async fn test_public_api_ping() {
        run_api_test(pool, || async {
            let client = HexstodyClient::new(&format!(
                "http://{}:{}",
                SERVICE_TEST_HOST, SERVICE_TEST_PORT
            ));
            client.ping().await.unwrap();
        })
        .await;
    }
}

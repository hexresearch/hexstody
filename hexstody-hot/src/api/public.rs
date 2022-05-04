use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

use rocket::fs::{relative, FileServer};
use rocket::response::content;
use rocket::serde::json::Json;
use rocket::{get, routes};
use rocket_dyn_templates::Template;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};

use super::api_types::*;
use hexstody_db::domain::currency::Currency;
use hexstody_db::state::State;
use hexstody_db::Pool;

#[openapi(tag = "ping")]
#[get("/ping")]
fn ping() -> content::Json<()> {
    content::Json(())
}

#[openapi(tag = "get_balance")]
#[get("/get_balance")]
fn get_balance() -> Json<Balance> {
    let x = Balance {
        balances: vec![BalanceItem {
            currency: Currency::BTC,
            value: 100,
        }],
    };

    Json(x)
}

#[openapi(tag = "get_history")]
#[get("/get_history")]
fn get_history() -> Json<History> {
    let x = History {
        history_items: vec![
            HistoryItem::Deposit(DepositHistoryItem {
                currency: Currency::BTC,
                value: 100,
            }),
            HistoryItem::Withdrawal(WithdrawalHistoryItem {
                currency: Currency::ETH,
                value: 300,
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

pub async fn serve_public_api(
    pool: Pool,
    state: Arc<Mutex<State>>,
    state_notify: Arc<Notify>,
    port: u16,
) -> Result<(), rocket::Error> {
    let figment = rocket::Config::figment().merge(("port", port));
    rocket::custom(figment)
        .mount("/static", FileServer::from(relative!("static/")))
        .mount("/", openapi_get_routes![ping, get_balance, get_history])
        .mount("/", routes![index, overview])
        .mount(
            "/swagger/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .attach(Template::fairing())
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

        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn({
            let state = state_mx.clone();
            let state_notify = state_notify.clone();
            async move {
                let serve_task = serve_public_api(pool, state, state_notify, SERVICE_TEST_PORT);
                futures::pin_mut!(serve_task);
                futures::future::select(serve_task, receiver.map_err(drop)).await;
            }
        });

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

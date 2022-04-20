use hexstody_db::Pool;
use hexstody_db::state::State;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

use rocket::{get, serde::json::Json};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use serde::{Deserialize, Serialize};
use rocket::Config;

use rocket::http::Status;
use rocket::response::{content, status};

#[openapi(tag = "ping")]
#[get("/ping")]
fn json() -> content::Json<()> {
    content::Json(())
}

pub async fn serve_public_api(pool: Pool, state: Arc<Mutex<State>>, state_notify: Arc<Notify> ) -> () {
  rocket::build()
    .mount("/", openapi_get_routes![json])
    .mount(
      "/swagger/",
      make_swagger_ui(&SwaggerUIConfig {
          url: "../openapi.json".to_owned(),
          ..Default::default()
      }))
     .launch().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexstody_client::client::HexstodyClient;
    use futures::FutureExt;
    use futures_util::future::TryFutureExt;
    use std::panic::AssertUnwindSafe;
    use futures::Future;

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
                let serve_task = serve_public_api(
                    pool,
                    state,
                    state_notify,
                );
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
        run_api_test(
            pool,
            || async {
                let client = HexstodyClient::new(&format!(
                    "http://{}:{}",
                    SERVICE_TEST_HOST, SERVICE_TEST_PORT
                ));
                client
                    .ping()
                    .await
                    .unwrap();
            },
        )
        .await;
    }
}

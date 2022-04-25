use rocket::get;
use rocket::response::content;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

#[openapi(tag = "ping")]
#[get("/ping")]
fn json() -> content::Json<()> {
    content::Json(())
}

pub async fn serve_public_api() -> () {
    rocket::build()
        .mount("/", openapi_get_routes![json])
        .mount(
            "/swagger/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .launch()
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;
    use futures::FutureExt;
    use futures_util::future::TryFutureExt;
    use hexstody_btc_client::client::BtcClient;
    use std::panic::AssertUnwindSafe;

    const SERVICE_TEST_PORT: u16 = 8000;
    const SERVICE_TEST_HOST: &str = "127.0.0.1";

    async fn run_api_test<F, Fut>(test_body: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        let _ = env_logger::builder().is_test(true).try_init();

        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn({
            async move {
                let serve_task = serve_public_api();
                futures::pin_mut!(serve_task);
                futures::future::select(serve_task, receiver.map_err(drop)).await;
            }
        });

        let res = AssertUnwindSafe(test_body()).catch_unwind().await;

        sender.send(()).unwrap();

        assert!(res.is_ok());
    }

    async fn test_public_api_ping() {
        run_api_test(|| async {
            let client = BtcClient::new(&format!(
                "http://{}:{}",
                SERVICE_TEST_HOST, SERVICE_TEST_PORT
            ));
            client.ping().await.unwrap();
        })
        .await;
    }
}

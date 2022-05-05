use super::types::*;
use rocket::figment::{providers::Env, Figment};
use rocket::{get, serde::json::Json, Config};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi, openapi_get_routes, rapidoc::*, swagger_ui::*};
use rocket::fairing::AdHoc;
use std::net::IpAddr;
use tokio::sync::Notify;
use std::sync::Arc;

#[openapi(tag = "misc")]
#[get("/ping")]
fn ping() -> Json<()> {
    Json(())
}

#[openapi(tag = "events")]
#[get("/deposit")]
async fn deposit() -> Json<DepositEvents> {
    Json(DepositEvents { events: vec![] })
}

pub async fn serve_public_api(address: IpAddr, port: u16, start_notify: Arc<Notify>) -> Result<(), rocket::Error> {
    let figment = Figment::from(Config {
        address,
        port,
        ..Config::default()
    })
    .merge(Env::prefixed("HEXSTODY_BTC_").global());

    let on_ready = AdHoc::on_liftoff("API Start!", |_| Box::pin(async move {
        start_notify.notify_one();
    }));

    rocket::custom(figment)
        .mount("/", openapi_get_routes![ping, deposit])
        .mount(
            "/swagger/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
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
    use hexstody_btc_client::client::BtcClient;
    use std::panic::AssertUnwindSafe;

    const SERVICE_TEST_PORT: u16 = 8289;
    const SERVICE_TEST_HOST: &str = "127.0.0.1";

    async fn run_api_test<F, Fut>(test_body: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        let _ = env_logger::builder().is_test(true).try_init();
        let start_notify = Arc::new(Notify::new());

        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn({
            let start_notify = start_notify.clone();
            async move {
                let serve_task =
                    serve_public_api(SERVICE_TEST_HOST.parse().unwrap(), SERVICE_TEST_PORT, start_notify);
                futures::pin_mut!(serve_task);
                futures::future::select(serve_task, receiver.map_err(drop)).await;
            }
        });
        start_notify.notified().await;
        
        let res = AssertUnwindSafe(test_body()).catch_unwind().await;

        sender.send(()).unwrap();

        assert!(res.is_ok());
    }

    #[tokio::test]
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

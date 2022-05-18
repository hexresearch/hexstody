use crate::state::ScanState;
use bitcoincore_rpc::{Client, RpcApi};
use bitcoincore_rpc_json::AddressType;
use hexstody_btc_api::bitcoin::*;
use hexstody_btc_api::events::*;
use log::*;
use rocket::fairing::AdHoc;
use rocket::figment::{providers::Env, Figment};
use rocket::{get, post, serde::json::Json, Config, State};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi, openapi_get_routes, rapidoc::*, swagger_ui::*};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use tokio::time::timeout;

use super::error;

#[openapi(tag = "misc")]
#[get("/ping")]
fn ping() -> Json<()> {
    Json(())
}

#[openapi(tag = "events")]
#[post("/events")]
async fn poll_events(
    polling_timeout: &State<Duration>,
    state: &State<Arc<Mutex<ScanState>>>,
    state_notify: &State<Arc<Notify>>,
) -> Json<BtcEvents> {
    trace!("Awaiting state events");
    match timeout(*polling_timeout.inner(), state_notify.notified()).await {
        Ok(_) => {
            info!("Got new events for deposit");
        }
        Err(_) => {
            trace!("No new events but releasing long poll");
        }
    }
    let mut state_rw = state.lock().await;
    let result = Json(BtcEvents {
        hash: state_rw.last_block.into(),
        height: state_rw.last_height,
        events: state_rw.events.clone(),
    });
    state_rw.events = vec![];
    result
}

#[openapi(tag = "deposit")]
#[post("/deposit/address")]
async fn get_deposit_address(client: &State<Client>) -> error::Result<BtcAddress> {
    let address = client
        .get_new_address(None, Some(AddressType::Bech32))
        .map_err(|e| error::Error::from(e))?;
    Ok(Json(address.into()))
}

pub async fn serve_public_api(
    btc: Client,
    address: IpAddr,
    port: u16,
    start_notify: Arc<Notify>,
    state: Arc<Mutex<ScanState>>,
    state_notify: Arc<Notify>,
    polling_duration: Duration,
    secret_key: &str,
) -> Result<(), rocket::Error> {
    let figment = Figment::from(Config {
        address,
        port,
        ..Config::default()
    })
    .merge(("secret_key", secret_key))
    .merge(Env::prefixed("HEXSTODY_BTC_").global());

    let on_ready = AdHoc::on_liftoff("API Start!", |_| {
        Box::pin(async move {
            start_notify.notify_one();
        })
    });

    rocket::custom(figment)
        .mount(
            "/",
            openapi_get_routes![ping, poll_events, get_deposit_address],
        )
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
        .manage(polling_duration)
        .manage(state)
        .manage(state_notify)
        .manage(btc)
        .attach(on_ready)
        .launch()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use hexstody_btc_test::runner::*;

    #[tokio::test]
    async fn test_public_api_ping() {
        run_test(|_, api| async move {
            api.ping().await.unwrap();
        })
        .await;
    }

    #[tokio::test]
    async fn test_public_api_address() {
        run_test(|_, api| async move {
            assert!(api.deposit_address().await.is_ok());
        })
        .await;
    }
}

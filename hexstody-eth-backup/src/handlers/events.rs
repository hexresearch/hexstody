#![allow(dead_code, unused_variables, non_snake_case)]
use crate::types::*;
use crate::node_calls;
use crate::db_functions;
use crate::conf::NodeConfig;
use log::*;


use rocket::{get,post,State};
use rocket::http::{Status, ContentType};
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use rocket_okapi::openapi;

use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tokio::time::timeout;

#[openapi(tag = "events")]
#[post("/events")]
async fn poll_events(
    polling_timeout: &State<Duration>,
    state: &State<Arc<Mutex<ScanState>>>,
    state_notify: &State<Arc<Notify>>,
) -> Json<EthEvents> {
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
    let result = Json(EthEvents {
        hash: state_rw.last_block.into(),
        height: state_rw.last_height,
        events: state_rw.events.clone(),
    });
    state_rw.events = vec![];
    result
}

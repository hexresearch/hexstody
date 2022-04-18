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
#[get("/")]
fn json() -> content::Json<()> {
    content::Json(())
}

pub async fn serve_public_api2(pool: Pool, state: Arc<Mutex<State>>, state_notify: Arc<Notify> ){
  print!("test");
  let t = 
    rocket::build()
      .mount("/", openapi_get_routes![json])
      .mount(
        "/swagger-ui/",
        make_swagger_ui(&SwaggerUIConfig {
            url: "../openapi.json".to_owned(),
            ..Default::default()
        }),
    )
       .launch().await;
       print!("test");
}
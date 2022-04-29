use rocket::fs::{relative, FileServer};
use rocket::{self, get, routes};
use rocket_dyn_templates::Template;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

use hexstody_db::state::State;
use hexstody_db::Pool;

#[openapi(skip)]
#[get("/")]
fn index() -> Template {
    let context = HashMap::from([("title", "Index"), ("parent", "base")]);
    Template::render("index", context)
}

pub async fn serve_operator_api(
    pool: Pool,
    state: Arc<Mutex<State>>,
    state_notify: Arc<Notify>,
    port: u16
) -> () {
    let figment = rocket::Config::figment().merge(("port", port));
    rocket::custom(figment)
        .mount("/", routes![index])
        .mount("/static", FileServer::from(relative!("static/")))
        .mount(
            "/swagger/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .attach(Template::fairing())
        .launch()
        .await;
}

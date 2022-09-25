use crate::types::*;
use crate::node_calls;
use crate::db_functions;
use crate::conf::{NodeConfig, load_config};

use rocket::{get,post,State};
use rocket::http::{Status, ContentType};
use rocket::serde::json::Json;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};

#[openapi(tag = "version")]
#[get("/version")]
pub fn getversion(cfg: &State<NodeConfig>) -> String {
    let res = node_calls::get_node_version(&cfg);
    return format!("Version is {:?}",res);
}

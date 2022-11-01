use crate::node_calls;
use crate::conf::NodeConfig;

use rocket::{get,State};
use rocket_okapi::openapi;

#[openapi(tag = "version")]
#[get("/version")]
pub fn getversion(cfg: &State<NodeConfig>) -> String {
    let res = node_calls::get_node_version(&cfg);
    return format!("Version is {:?}",res);
}

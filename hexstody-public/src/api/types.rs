use serde::{Deserialize, Serialize};
use std::i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserETH {
    pub id: i32,
    pub login: String,
    pub address: String,
}

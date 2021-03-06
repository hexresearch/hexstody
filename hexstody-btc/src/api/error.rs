use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to query BTC node: {0}")]
    NodeRpc(#[from] bitcoincore_rpc::Error),
    #[error("Debug endpoints are enabled only for regtest")]
    DebugNotEnabled,
    #[error("Failed to parse address: {0}")]
    AddressParse(#[from] bitcoin::util::address::Error),
}

impl Error {
    pub fn code(&self) -> u16 {
        match self {
            Error::NodeRpc(_) => 0,
            Error::DebugNotEnabled => 1,
            Error::AddressParse(_) => 2,
        }
    }

    pub fn status(&self) -> Status {
        match self {
            Error::NodeRpc(_) => Status::from_code(500).unwrap(),
            Error::DebugNotEnabled => Status::from_code(500).unwrap(),
            Error::AddressParse(_) => Status::from_code(400).unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct ErrorMessage {
    pub message: String,
    pub code: u16,
}

pub type Result<T> = std::result::Result<Json<T>, (Status, Json<ErrorMessage>)>;

impl From<Error> for ErrorMessage {
    fn from(value: Error) -> Self {
        ErrorMessage {
            message: format!("{value}"),
            code: value.code(),
        }
    }
}

impl From<Error> for (Status, Json<ErrorMessage>) {
    fn from(value: Error) -> Self {
        (value.status(), Json(value.into()))
    }
}

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to sign up new user. The user already exists.")]
    SignupExistedUser,
}

impl Error {
    pub fn code(&self) -> u16 {
        match self {
            Error::SignupExistedUser => 0,
        }
    }

    pub fn status(&self) -> Status {
        match self {
            Error::SignupExistedUser => Status::from_code(401).unwrap(),
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

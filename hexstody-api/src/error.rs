use crate::domain::currency::Currency;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MIN_USER_NAME_LEN: usize = 3;
pub const MAX_USER_NAME_LEN: usize = 320;
pub const MIN_USER_PASSWORD_LEN: usize = 6;
pub const MAX_USER_PASSWORD_LEN: usize = 1024;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to sign up new user. The user already exists.")]
    SignupExistedUser,
    #[error(
        "Failed to signup user. The user name is too short. Need >= {MIN_USER_NAME_LEN} symbols"
    )]
    UserNameTooShort,
    #[error(
        "Failed to signup user. The user name is too long. Need <= {MAX_USER_NAME_LEN} symbols"
    )]
    UserNameTooLong,
    #[error("Failed to signup user. The user password is too short. Need >= {MIN_USER_PASSWORD_LEN} symbols")]
    UserPasswordTooShort,
    #[error("Failed to signup user. The user password is too long. Need <= {MAX_USER_PASSWORD_LEN} symbols")]
    UserPasswordTooLong,
    #[error("Password hash failed: {0}")]
    Pwhash(#[from] pwhash::error::Error),
    #[error("Username of password is invalid")]
    SigninFailed,
    #[error("Action requires authentification")]
    AuthRequired,
    #[error("Authed user is not found in state!")]
    NoUserFound,
    #[error("Authed user doesn't have required currency {0}!")]
    NoUserCurrency(Currency),
}

impl Error {
    pub fn code(&self) -> u16 {
        match self {
            Error::SignupExistedUser => 0,
            Error::UserNameTooShort => 1,
            Error::UserNameTooLong => 2,
            Error::UserPasswordTooShort => 3,
            Error::UserPasswordTooLong => 4,
            Error::Pwhash(_) => 5,
            Error::SigninFailed => 6,
            Error::AuthRequired => 7,
            Error::NoUserFound => 8,
            Error::NoUserCurrency(_) => 9,
        }
    }

    pub fn status(&self) -> Status {
        match self {
            Error::SignupExistedUser => Status::from_code(400).unwrap(),
            Error::UserNameTooShort => Status::from_code(400).unwrap(),
            Error::UserNameTooLong => Status::from_code(400).unwrap(),
            Error::UserPasswordTooShort => Status::from_code(400).unwrap(),
            Error::UserPasswordTooLong => Status::from_code(400).unwrap(),
            Error::Pwhash(_) => Status::from_code(500).unwrap(),
            Error::SigninFailed => Status::from_code(401).unwrap(),
            Error::AuthRequired => Status::from_code(401).unwrap(),
            Error::NoUserFound => Status::from_code(500).unwrap(),
            Error::NoUserCurrency(_) => Status::from_code(500).unwrap(),
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

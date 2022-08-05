use crate::domain::{currency::Currency, Erc20Token};
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
    #[error("Failed to generate new deposit address for currency {0}")]
    FailedGenAddress(Currency),
    #[error("Failed to get fee for currency {0}")]
    FailedGetFee(Currency),
    #[error("Not enough {0}!")]
    InsufficientFunds(Currency),
    #[error("Failed to connect to ETH node: {0}")]
    FailedETHConnection(String),
    #[error("{0} is already enabled")]
    TokenAlreadyEnabled(Erc20Token),
    #[error("{0} is already disabled")]
    TokenAlreadyDisabled(Erc20Token),
    #[error("{0} has non-zero balance. Can not disable")]
    TokenNonZeroBalance(Erc20Token),
    #[error("Token action failed: {0}")]
    TokenActionFailed(String)
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
            Error::FailedGenAddress(_) => 10,
            Error::FailedGetFee(_) => 11,
            Error::InsufficientFunds(_) => 12,
            Error::FailedETHConnection(_) => 13,
            Error::TokenAlreadyEnabled(_) => 14,
            Error::TokenAlreadyDisabled(_) => 15,
            Error::TokenNonZeroBalance(_) => 16,
            Error::TokenActionFailed(_) => 17,
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
            Error::NoUserFound => Status::from_code(417).unwrap(),
            Error::NoUserCurrency(_) => Status::from_code(500).unwrap(),
            Error::FailedGenAddress(_) => Status::from_code(500).unwrap(),
            Error::FailedGetFee(_) => Status::from_code(500).unwrap(),
            Error::InsufficientFunds(_) =>  Status::from_code(417).unwrap(),
            Error::FailedETHConnection(_) => Status::from_code(500).unwrap(),
            Error::TokenAlreadyEnabled(_) => Status::from_code(500).unwrap(),
            Error::TokenAlreadyDisabled(_) => Status::from_code(500).unwrap(),
            Error::TokenNonZeroBalance(_) => Status::from_code(500).unwrap(),
            Error::TokenActionFailed(_) => Status::from_code(500).unwrap()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct ErrorMessage {
    pub message: String,
    pub code: u16,
}

pub type Result<T> = std::result::Result<T, (Status, Json<ErrorMessage>)>;

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

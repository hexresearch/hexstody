use thiserror::Error;
use crate::domain::{currency::Currency, Erc20Token};
pub use crate::error::HexstodyError;
pub use crate::error::Result;
pub use crate::error::ErrorMessage;
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
    TokenActionFailed(String),
    #[error("Invite does not exist")]
    InviteNotFound,
    #[error("Limits are not changed by the update")]
    LimitsNoChanges,
    #[error("Limit change not found")]
    LimChangeNotFound,
    #[error("Signature error: {0}")]
    SignatureError(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Error: {0}")]
    GenericError(String),
    #[error("Unknown currency: {0}")]
    UnknownCurrency(String),
    #[error("Language is not changed!")]
    LangNotChanged,
    #[error("Invalid e-mail")]
    InvalidEmail,
    #[error("Invalid phone number")]
    InvalidPhoneNumber,
    #[error("Failed to get {0} exchange rate")]
    ExchangeRateError(Currency),
    #[error("Malformed margin: {0}")]
    MalformedMargin(String),
}

impl HexstodyError for Error {
    fn subtype() -> &'static str {
        "hexstody_api"
    }
    fn code(&self) -> u16 {
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
            Error::InviteNotFound => 18,
            Error::LimitsNoChanges => 19,
            Error::LimChangeNotFound => 20,
            Error::SignatureError(_) => 21,
            Error::UnknownCurrency(_) => 22,
            Error::InternalServerError(_) => 23,
            Error::GenericError(_) => 24,
            Error::LangNotChanged => 25,
            Error::InvalidEmail => 26,
            Error::InvalidPhoneNumber => 27,
            Error::ExchangeRateError(_) => 28,
            Error::MalformedMargin(_) => 29,
        }
    }

    fn status(&self) -> u16 {
        match self {
            Error::SignupExistedUser => 400,
            Error::UserNameTooShort => 400,
            Error::UserNameTooLong => 400,
            Error::UserPasswordTooShort => 400,
            Error::UserPasswordTooLong => 400,
            Error::Pwhash(_) => 500,
            Error::SigninFailed => 401,
            Error::AuthRequired => 401,
            Error::NoUserFound => 417,
            Error::NoUserCurrency(_) => 500,
            Error::FailedGenAddress(_) => 500,
            Error::FailedGetFee(_) => 500,
            Error::InsufficientFunds(_) => 417,
            Error::FailedETHConnection(_) => 500,
            Error::TokenAlreadyEnabled(_) => 500,
            Error::TokenAlreadyDisabled(_) => 500,
            Error::TokenNonZeroBalance(_) => 500,
            Error::TokenActionFailed(_) => 500,
            Error::InviteNotFound => 400,
            Error::LimitsNoChanges => 500,
            Error::LimChangeNotFound => 400,
            Error::SignatureError(_) => 403,
            Error::InternalServerError(_) => 500,
            Error::GenericError(_) => 500,
            Error::UnknownCurrency(_) => 400,
            Error::LangNotChanged => 400,
            Error::InvalidEmail => 400,
            Error::InvalidPhoneNumber => 400,
            Error::ExchangeRateError(_) => 404,
            Error::MalformedMargin(_) => 400,
        }
    }
}
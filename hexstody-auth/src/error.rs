use hexstody_api::error::HexstodyError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Action requires authentification")]
    AuthRequired,
    #[error("Authed user is not found in state!")]
    NoUserFound,
}

impl HexstodyError for Error {
    fn subtype() -> &'static str {
        "hexstody_auth"
    }

    fn code(&self) -> u16 {
        match self {
            Error::AuthRequired => 0,
            Error::NoUserFound => 1,
        }
    }

    fn status(&self) -> u16 {
        match self {
            Error::AuthRequired => 403,
            Error::NoUserFound => 403,
        }
    }
}
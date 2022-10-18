use thiserror::Error;
pub use hexstody_api::error::HexstodyError;
pub use hexstody_api::error::Result;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Generic error: {0}")]
    GenericError(String)
}

impl HexstodyError for Error {
    fn subtype() -> &'static str {
        "invoice_api"
    }
    fn code(&self) -> u16 {
        match self {
            Error::GenericError(_) => 0,
        }
    }

    fn status(&self) -> u16 {
        match self {
            Error::GenericError(_) => 500,
        }
    }
}
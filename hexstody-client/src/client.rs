use log::*;
use thiserror::Error;
use hexstody_api::types::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwesting server error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON encoding/decoding error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Alias for a `Result` with the error type `self::Error`.
pub type Result<T> = std::result::Result<T, Error>;

pub struct HexstodyClient {
    pub client: reqwest::Client,
    pub server: String,
}

impl HexstodyClient {
    pub fn new(url: &str) -> Self {
        HexstodyClient {
            client: reqwest::Client::new(),
            server: url.to_owned(),
        }
    }

    pub async fn ping(&self) -> Result<()> {
        let path = "/ping";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.get(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(())
    }

    pub async fn signup_email(&self, data: SignupEmail) -> Result<()> {
        let path = "/signup/email";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).json(&data).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(())
    }

    pub async fn signin_email(&self, data: SigninEmail) -> Result<()> {
        let path = "/signin/email";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).json(&data).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(())
    }
}

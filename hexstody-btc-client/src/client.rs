use log::*;
use thiserror::Error;
use hexstody_btc_api::deposit::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwesting server error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON encoding/decoding error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Alias for a `Result` with the error type `self::Error`.
pub type Result<T> = std::result::Result<T, Error>;

pub struct BtcClient {
    pub client: reqwest::Client,
    pub server: String,
}

impl BtcClient {
    pub fn new(url: &str) -> Self {
        BtcClient {
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
        debug!("Response ping: {}", response);
        Ok(())
    }

    pub async fn deposit_events(&self) -> Result<DepositEvents> {
        let path = "/events/deposit";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response events/deposit: {}", response);
        Ok(serde_json::from_str(&response)?)
    }
}

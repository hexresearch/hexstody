use bitcoin::Address;
use hexstody_api::types::FeeResponse;
use hexstody_btc_api::events::*;
use log::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Requesting server error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON encoding/decoding error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Alias for a `Result` with the error type `self::Error`.
pub type Result<T> = std::result::Result<T, Error>;

pub const BTC_BYTES_PER_TRANSACTION: u64 = 1024;

#[derive(Clone)]
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
        debug!("Response {path}: {}", response);
        Ok(())
    }

    pub async fn poll_events(&self) -> Result<BtcEvents> {
        let path = "/events";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(serde_json::from_str(&response)?)
    }

    pub async fn deposit_address(&self) -> Result<Address> {
        let path = "/deposit/address";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(serde_json::from_str(&response)?)
    }

    pub async fn get_fees(&self) -> Result<FeeResponse> {
        let path = "/fees";
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
        Ok(serde_json::from_str(&response)?)
    }
}

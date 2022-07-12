use hexstody_api::domain::*;
use hexstody_api::types::*;
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

#[derive(Clone)]
pub struct HexstodyClient {
    pub client: reqwest::Client,
    pub server: String,
}

impl HexstodyClient {
    pub fn new(url: &str) -> reqwest::Result<Self> {
        Ok(HexstodyClient {
            client: reqwest::ClientBuilder::new().cookie_store(true).build()?,
            server: url.to_owned(),
        })
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

    pub async fn logout(&self) -> Result<()> {
        let path = "/logout";
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
        Ok(())
    }

    pub async fn get_balance(&self) -> Result<Balance> {
        let path = "/balance";
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

    pub async fn get_deposit(&self, currency: Currency) -> Result<DepositInfo> {
        let path = "/deposit";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).json(&currency).build()?;
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

    pub async fn get_deposit_eth(&self, currency: Currency) -> Result<DepositInfo> {
        let path = "/depositETH";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).json(&currency).build()?;
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
    
    pub async fn eth_ticker(&self, currency: Currency) -> Result<TickerETH> {
        let path = "/ethticker";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).json(&currency).build()?;
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

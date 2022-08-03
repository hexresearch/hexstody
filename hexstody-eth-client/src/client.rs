use hexstody_api::{domain::Erc20Token, types::UserEth};
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
pub struct EthClient {
    pub client: reqwest::Client,
    pub server: String,
}

impl EthClient {
    pub fn new(url: &str) -> Self {
        EthClient {
            client: reqwest::Client::new(),
            server: url.to_owned(),
        }
    }

    pub async fn createuser(&self, user: &str) -> Result<()> {
        let path = "/createuser";
        let endpoint = format!("{}{}/{}", self.server, path, user);
        let request = self.client.get(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?;
        debug!("Response {path}: {:?}", response);
        Ok(())
    }

    pub async fn post_tokens(&self, user: &str, tokens: &Vec<Erc20Token>) -> Result<()> {
        let path = "/tokens";
        let endpoint = format!("{}{}/{}", self.server, path, user);
        let request = self.client.post(endpoint).json(tokens).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {:?}", response);
        Ok(())
    }

    pub async fn get_user_data(&self, user: &str) -> Result<UserEth> {
        let path = "/userdata";
        let endpoint = format!("{}{}/{}", self.server, path, user);
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

    pub async fn send_tx(&self, addr: &str, amount: &str) -> Result<()>{
        let path = "/sendtx";
        let endpoint = format!("{}{}/{}/{}", self.server, path, addr, amount);
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

    pub async fn send_token_tx(&self, token: &Erc20Token, addr: &str, amount: &str) -> Result<()>{
        let path = "/sendtokentx";
        let endpoint = format!("{}{}/{}/{}/{}", self.server, path, &token.contract, addr, amount);
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

    pub async fn remove_user(&self, user: &str) -> Result<()> {
        let path = "/removeuser";
        let endpoint = format!("{}{}/{}", self.server, path, user);
        let request = self.client.get(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status();
        debug!("Response {path}: {:?}", response);
        Ok(())
    }
}
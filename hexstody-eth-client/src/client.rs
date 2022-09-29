use hexstody_api::{
    domain::{Currency, Erc20Token},
    types::{
        Erc20HotWalletBalanceResponse, EthHotWalletBalanceResponse, HotBalanceResponse, UserEth,
    },
};
use log::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Requesting server error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON encoding/decoding error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid currency: {0}")]
    InvalidCurrency(Currency),
    #[error("Currency not found: {0}")]
    CurrencyNotFound(Currency),
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
        let response = self.client.execute(request).await?.error_for_status()?;
        debug!("Response {path}: {:?}", response);
        Ok(())
    }

    pub async fn allocate_address(&self, user: &str) -> Result<String> {
        let path = "/check_address";
        let endpoint = format!("{}{}/{}", self.server, path, user);
        let request = self.client.get(endpoint).build()?;
        let response = self.client.execute(request)
            .await?
            .error_for_status()?
            .json()
            .await?;
        debug!("Response {path}: {:?}", response);
        Ok(response)
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

    pub async fn send_tx(&self, user: &str, addr: &str, amount: &str) -> Result<()> {
        let path = "/signingsend/eth/login/";
        let endpoint = format!("{}{}/{}/{}/{}", self.server, path, user, addr, amount);
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

    pub async fn send_tx_erc20(&self
                            , user: &str
                            , addr: &str
                            , token_addr: &str
                            , amount: &str) -> Result<()> {
        let path = "/signingsend/erc20/login/";
        let endpoint = format!("{}{}/{}/{}/{}/{}"
                            , self.server
                            , path
                            , user
                            , addr
                            , token_addr
                            , amount);
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

    pub async fn send_token_tx(&self, token: &Erc20Token, addr: &str, amount: &str) -> Result<()> {
        let path = "/sendtokentx";
        let endpoint = format!(
            "{}{}/{}/{}/{}",
            self.server, path, &token.contract, addr, amount
        );
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
        let response = self.client.execute(request).await?.error_for_status();
        debug!("Response {path}: {:?}", response);
        Ok(())
    }

    pub async fn get_hot_wallet_balance(&self, currency: &Currency) -> Result<HotBalanceResponse> {
        let path = match currency {
            Currency::ETH => "/balance/eth/total",
            Currency::ERC20(_) => "/balance/erc20/total",
            Currency::BTC => return Err(Error::InvalidCurrency(Currency::BTC)),
        };
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.get(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {:?}", response);
        match currency {
            Currency::ETH => {
                let eth_balance: EthHotWalletBalanceResponse = serde_json::from_str(&response)?;
                return Ok(HotBalanceResponse {
                    balance: eth_balance.balance,
                });
            }
            Currency::ERC20(token) => {
                let erc20_balance: Erc20HotWalletBalanceResponse = serde_json::from_str(&response)?;
                let result = erc20_balance
                    .balance
                    .into_iter()
                    .find(|x| x.token_name == token.ticker);
                match result {
                    None => return Err(Error::CurrencyNotFound(currency.clone())),
                    Some(b) => {
                        return Ok(HotBalanceResponse {
                            balance: b.token_balance,
                        })
                    }
                };
            }
            Currency::BTC => return Err(Error::InvalidCurrency(Currency::BTC)),
        };
    }
}

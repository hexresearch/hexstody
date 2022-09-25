use std::{collections::HashMap, fmt::Debug};

use hexstody_api::domain::Symbol;
use log::*;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Requesting server error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON encoding/decoding error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0} not found in response")]
    ResponseMissing(String),
    #[error("Generic error: {0}")]
    GenericError(String)
}

/// Alias for a `Result` with the error type `self::Error`.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct TickerClient {
    pub client: reqwest::Client,
    pub server: String,
}

impl TickerClient {
    pub fn new(url: &str) -> Self {
        TickerClient {
            client: reqwest::Client::new(),
            server: url.to_owned(),
        }
    }

    /// Symbol to Symbol ticker 
    pub async fn symbol_to_symbol(&self, from: &Symbol, to: &Symbol) -> Result<f64> {
        let path = "data/price";
        let endpoint = format!("{}/{}?fsym={}&tsyms={}",self.server, path, from.symbol(), to.symbol());
        let request = self.client.get(endpoint).build()?;
        let response = self.client.execute(request)
            .await?
            .error_for_status()?
            .json::<HashMap<String, f64>>()
            .await?;
        debug!("Response {path}: {:?}", response);
        response
            .get(&to.symbol())
            .ok_or(Error::ResponseMissing(to.symbol()))
            .cloned()
    }

    /// Concrete symbol to many symbols request.
    pub async fn symbol_to_symbols(&self, from: &Symbol, to: &Vec<Symbol>) -> Result<HashMap<Symbol, f64>>
    {
        let tsyms = to.iter().map(|f| f.symbol()).collect::<Vec<String>>().join(",");
        let path = "data/price";
        let endpoint = format!("{}/{}?fsym={}&tsyms={}",self.server, path, from.symbol(), tsyms);
        let request = self.client.get(endpoint).build()?;
        let response = self.client.execute(request)
            .await?
            .error_for_status()?
            .json()
            .await?;
        debug!("Response {path}: {:?}", response);
        Ok(response)
    }

    /// Get multiple values: Symbol to multiple Symbols with generic return. Catch all is HashMap<String, f64>
    pub async fn symbol_to_symbols_generic<T>(&self, from: &Symbol, to: &Vec<Symbol>) -> Result<T>
    where T: DeserializeOwned + Debug
    {
        let tsyms = to.iter().map(|f| f.symbol()).collect::<Vec<String>>().join(",");
        let path = "data/price";
        let endpoint = format!("{}/{}?fsym={}&tsyms={}",self.server, path, from.symbol(), tsyms);
        let request = self.client.get(endpoint).build()?;
        let response = self.client.execute(request)
            .await?
            .error_for_status()?
            .json::<T>()
            .await?;
        debug!("Response {path}: {:?}", response);
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use std::{future::Future, panic::AssertUnwindSafe};
    use futures::FutureExt;
    use hexstody_api::types::TickerETH;
    use super::*;
    async fn run_test<F, Fut>(test_body: F)
    where
        F: FnOnce(TickerClient) -> Fut,
        Fut: Future<Output = ()>
    {
        let _ = env_logger::builder().is_test(true).try_init();
        let domain = "https://min-api.cryptocompare.com";
        let client = TickerClient::new(domain);
        let res = AssertUnwindSafe(test_body(client))
            .catch_unwind()
            .await;
        assert!(res.is_ok());
    }
    
    #[tokio::test]
    async fn test_btc_to_usd() {
        run_test(|client| async move {
            let resp = client.symbol_to_symbol(&Symbol::BTC, &Symbol::USD).await;
            assert!(resp.is_ok());
        }).await;
    }

    #[tokio::test]
    async fn test_eth_to_ethticker() {
        run_test(|client| async move {
            let resp = client.symbol_to_symbols_generic::<TickerETH>(&Symbol::ETH, &vec![Symbol::USD, Symbol::RUB]).await;
            assert!(resp.is_ok());
        }).await;
    }

    #[tokio::test]
    async fn test_eth_to_ethticker_fail() {
        run_test(|client| async move {
            let resp = client.symbol_to_symbols_generic::<TickerETH>(&Symbol::ETH, &vec![Symbol::USD]).await;
            assert!(resp.is_err());
        }).await;
    }

    #[tokio::test]
    async fn test_btc_to_btc() {
        run_test(|client| async move {
            let resp = client.symbol_to_symbol(&Symbol::BTC, &Symbol::BTC).await;
            assert!(resp.is_ok());
            let v = resp.unwrap();
            assert_eq!(v, 1.0);
        }).await;
    }
}

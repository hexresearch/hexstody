use std::{collections::HashMap, fmt::Debug};

use hexstody_api::{
    domain::{Currency, Fiat}
};
use log::*;
use serde::de::DeserializeOwned;
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

    /// Get crypto's price in a single fiat currency
    pub async fn fiat_ticker(&self, crypto: Currency, fiat: Fiat) -> Result<f64> {
        let path = "data/price";
        let endpoint = format!("{}/{}?fsym={}&tsyms={}",self.server, path, crypto.ticker(), fiat.ticker());
        let request = self.client.get(endpoint).build()?;
        let response = self.client.execute(request)
            .await?
            .error_for_status()?
            .json::<HashMap<String, f64>>()
            .await?;
        debug!("Response {path}: {:?}", response);
        response
            .get(&fiat.ticker())
            .ok_or(Error::ResponseMissing(fiat.ticker()))
            .cloned()
    }

    /// Get multiple fiat tickers
    /// Return type is generic since only the caller knows it
    /// Catch-all type is HashMap<String, f64>
    pub async fn multi_fiat_ticker<T> (&self, crypto: Currency, fiats: Vec<Fiat>) -> Result<T>
    where T: DeserializeOwned + Debug
    {
        let tsyms = fiats.iter().map(|f| f.ticker()).collect::<Vec<String>>().join(",");
        let path = "data/price";
        let endpoint = format!("{}/{}?fsym={}&tsyms={}",self.server, path, crypto.ticker(), tsyms);
        let request = self.client.get(endpoint).build()?;
        let response = self.client.execute(request)
            .await?
            .error_for_status()?
            .json::<T>()
            .await?;
        debug!("Response {path}: {:?}", response);
        Ok(response)
    }

    /// Get a ticker to a pair of cryptos
    pub async fn ticker_pair(&self, from: Currency, to: Currency) -> Result<f64> {
        let path = "data/price";
        let endpoint = format!("{}/{}?fsym={}&tsyms={}",self.server, path, from.ticker(), to.ticker());
        let request = self.client.get(endpoint).build()?;
        let response = self.client.execute(request)
            .await?
            .error_for_status()?
            .json::<HashMap<String, f64>>()
            .await?;
        debug!("Response {path}: {:?}", response);
        response
            .get(&to.ticker())
            .ok_or(Error::ResponseMissing(to.ticker()))
            .cloned()
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
            let resp = client.fiat_ticker(Currency::BTC, Fiat::USD).await;
            assert!(resp.is_ok());
        }).await;
    }

    #[tokio::test]
    async fn test_eth_to_ethticker() {
        run_test(|client| async move {
            let resp = client.multi_fiat_ticker::<TickerETH>(Currency::ETH, vec![Fiat::USD, Fiat::RUB]).await;
            assert!(resp.is_ok());
        }).await;
    }

    #[tokio::test]
    async fn test_eth_to_ethticker_fail() {
        run_test(|client| async move {
            let resp = client.multi_fiat_ticker::<TickerETH>(Currency::ETH, vec![Fiat::USD]).await;
            assert!(resp.is_err());
        }).await;
    }

    #[tokio::test]
    async fn test_btc_to_btc() {
        run_test(|client| async move {
            let resp = client.ticker_pair(Currency::BTC, Currency::BTC).await;
            assert!(resp.is_ok());
            let v = resp.unwrap();
            assert_eq!(v, 1.0);
        }).await;
    }
}

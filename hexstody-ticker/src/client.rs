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
        let path = "/data/price";
        let endpoint = format!("{}/{}?fsym={}&tsyms={}",self.server, path, crypto.ticker(), fiat.ticker());
        let request = self.client.get(endpoint).build()?;
        let response = self.client.execute(request)
            .await?
            .error_for_status()?
            .json::<HashMap<String, f64>>()
            .await?;
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
        let path = "/data/price";
        let endpoint = format!("{}/{}?fsym={}&tsyms={}",self.server, path, crypto.ticker(), tsyms);
        let request = self.client.get(endpoint).build()?;
        let response = self.client.execute(request)
            .await?
            .error_for_status()?
            .json::<T>()
            .await
            .map_err(|e| e.into());
        debug!("Response {path}: {:?}", response);
        response
    }

    /// Get a ticker to a pair of cryptos
    pub async fn ticker_pair(&self, from: Currency, to: Currency) -> Result<f64> {
        let path = "/data/price";
        let endpoint = format!("{}/{}?fsym={}&tsyms={}",self.server, path, from.ticker(), to.ticker());
        let request = self.client.get(endpoint).build()?;
        let response = self.client.execute(request)
            .await?
            .error_for_status()?
            .json::<HashMap<String, f64>>()
            .await?;
        response
            .get(&to.ticker())
            .ok_or(Error::ResponseMissing(to.ticker()))
            .cloned()
    }
}

// pub async fn run_test<F, Fut>(test_body: F)
// where
//     F: FnOnce(Client, BtcClient) -> Fut,
//     Fut: Future<Output = ()>,
// {
//     let _ = env_logger::builder().is_test(true).try_init();
//     let node_port = random_free_tcp_port().expect("available port");
//     let node_rpc_port = random_free_tcp_port().expect("available port");
//     let (node_handle, client, _tmp_dir) = setup_node_ready(node_port, node_rpc_port).await;
//     let api_port = setup_api(node_rpc_port).await;
//     info!("Running API server on {api_port}");
//     let api_client = BtcClient::new(&format!("http://127.0.0.1:{api_port}"));
//     let res = AssertUnwindSafe(test_body(client, api_client))
//         .catch_unwind()
//         .await;
//     teardown_node(node_handle);
//     assert!(res.is_ok());
// }
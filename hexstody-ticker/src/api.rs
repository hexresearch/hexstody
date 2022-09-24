use std::sync::Arc;

use hexstody_api::{
    types::TickerETH,
    domain::{Currency, Fiat}};
use hexstody_api::error;
use hexstody_runtime_db::RuntimeState;
use hexstody_ticker_provider::client::TickerClient;
use rocket::{post, State, Route, serde::json::Json};
use rocket_okapi::{openapi, openapi_get_routes, JsonSchema};
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;

pub fn ticker_api() -> Vec<Route> {
    openapi_get_routes![
        ticker,
        ticker_pair
    ]
}

#[openapi(tag = "wallet")]
#[post("/ticker", data = "<currency>")]
pub async fn ticker(
    rstate: &State<Arc<Mutex<RuntimeState>>>,
    ticker_client: &State<TickerClient>,
    currency: Json<Currency>,
) -> error::Result<Json<TickerETH>> {
    let currency = currency.into_inner();
    let mut rstate = rstate.lock().await;
    let ticker: TickerETH = rstate
        .get_multifiat_ticker(ticker_client, currency, vec![Fiat::USD, Fiat::RUB])
        .await
        .map_err(|e| error::Error::GenericError(e.to_string()))?;
    Ok(Json(ticker))
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CurrencyPair {
    from: Currency,
    to: Currency
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CurrencyPairResponse{
    from: Currency,
    to: Currency,
    rate: f64
}

#[openapi(tag = "wallet")]
#[post("/pair", data = "<currency_pair>")]
pub async fn ticker_pair(
    rstate: &State<Arc<Mutex<RuntimeState>>>,
    ticker_client: &State<TickerClient>,
    currency_pair: Json<CurrencyPair>,
) -> error::Result<Json<CurrencyPairResponse>> {
    let CurrencyPair{ from, to } = currency_pair.into_inner();
    let mut rstate = rstate.lock().await;
    let rate = rstate
        .get_pair_ticker(ticker_client, from.clone(), to.clone())
        .await
        .map_err(|e| error::Error::GenericError(e.to_string()))?;
    Ok(Json(CurrencyPairResponse{ from, to, rate }))
}

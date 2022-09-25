use std::sync::Arc;

use hexstody_api::{
    types::TickerETH,
    domain::{Currency, Symbol}};
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
        .symbol_to_symbols_generic(ticker_client, currency.symbol(), vec![Symbol::USD, Symbol::RUB])
        .await
        .map_err(|e| error::Error::GenericError(e.to_string()))?;
    Ok(Json(ticker))
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SymbolPair {
    from: Symbol,
    to: Symbol,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SymbolPairResponse{
    from: Symbol,
    to: Symbol,
    rate: f64
}

#[openapi(tag = "wallet")]
#[post("/pair", data = "<currency_pair>")]
pub async fn ticker_pair(
    rstate: &State<Arc<Mutex<RuntimeState>>>,
    ticker_client: &State<TickerClient>,
    currency_pair: Json<SymbolPair>,
) -> error::Result<Json<SymbolPairResponse>> {
    let SymbolPair{ from, to } = currency_pair.into_inner();
    let mut rstate = rstate.lock().await;
    let rate = rstate
        .symbol_to_symbol(ticker_client, from.clone(), to.clone())
        .await
        .map_err(|e| error::Error::GenericError(e.to_string()))?;
    Ok(Json(SymbolPairResponse{ from, to, rate }))
}

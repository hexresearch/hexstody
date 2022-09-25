use std::{sync::Arc, collections::HashMap};

use hexstody_runtime_db::RuntimeState;
use hexstody_ticker_provider::client::TickerClient;
use log::{debug, info};
use tokio::sync::Mutex;

/// Delay between refreshes in seconds
static REFRESH_PERIOD: u64 = 30;

pub async fn ticker_worker(
    ticker_client: TickerClient,
    rstate: Arc<Mutex<RuntimeState>>
){
    info!("Started ticker worker with period {}s", REFRESH_PERIOD);
    let mut period = tokio::time::interval(tokio::time::Duration::from_secs(REFRESH_PERIOD));
    loop {
        period.tick().await;
        let pairs = {rstate.lock().await.tracked_pairs()};
        let mut new_cache = HashMap::new();
        for (from, to) in pairs.into_iter() {
            let resp = ticker_client.symbol_to_symbols(&from, &to).await;
            match resp {
                Err(e) => debug!("Error requesting tickers: {}", e.to_string()),
                Ok(vals) => {new_cache.insert(from, vals);}
            }
        }
        let mut rstate = rstate.lock().await;
        rstate.cached_tickers = new_cache;
    }
} 

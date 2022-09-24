use std::collections::HashMap;
use std::fmt::Debug;

use hexstody_api::domain::{Currency, Fiat};
use hexstody_ticker_provider::client::TickerClient;
use hexstody_ticker_provider::client::Result as TickerResult;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::Map;

pub struct RuntimeState {
    /// Runtime cache of challenges to log in with a key
    pub challenges: HashMap<String, String>,
    /// Cached ticker info
    /// We store tikers in the string format: "BTC:ETH" etc
    /// since we want to uniformely store both Crypto and Fiat tickers in the same map 
    pub cached_tickers: HashMap<String, HashMap<String, f64>>
}

impl RuntimeState {
    pub fn new() -> Self{
        RuntimeState{
            challenges: HashMap::new(),
            cached_tickers: HashMap::new()
        }
    }

    pub async fn get_fiat_ticker(&mut self, client: &TickerClient, crypto: Currency, fiat: Fiat) -> TickerResult<f64>{
        let ct = crypto.ticker();
        let ft = fiat.ticker();
        let mrate = self.cached_tickers.get(&ct).map(|sm| sm.get(&ft)).flatten();
        match mrate {
            Some(rate) => Ok(rate.clone()),
            None => {
                let rate = client.fiat_ticker(crypto, fiat).await?;
                self.cached_tickers.entry(ct).and_modify(|cm| {cm.insert(ft, rate);});
                Ok(rate)
            },
        }
    }

    pub async fn get_pair_ticker(&mut self, client: &TickerClient, from: Currency, to: Currency) -> TickerResult<f64> {
        let ft = from.ticker();
        let tt = to.ticker();
        let mrate = self.cached_tickers.get(&ft).map(|sm| sm.get(&tt)).flatten();
        match mrate {
            Some(rate) => Ok(rate.clone()),
            None => {
                let rate = client.ticker_pair(from, to).await?;
                self.cached_tickers.entry(ft).and_modify(|cm| {cm.insert(tt, rate);});
                Ok(rate)
            },
        }
    }

    pub async fn get_multifiat_ticker<T>(&mut self, client: &TickerClient, crypto: Currency, fiats: Vec<Fiat>) -> TickerResult<T>
    where T: DeserializeOwned + Debug
    {
        let mut vals: Map<String, Value> = Map::new();
        let mut missing: Vec<Fiat> = vec![];
        let ct = crypto.ticker();
        let submap = self.cached_tickers.get(&ct);
        match submap {
            None => {
                let res: HashMap<String, f64> = client.multi_fiat_ticker(crypto.clone(), fiats).await?;
                self.cached_tickers.insert(ct.clone(), res.clone());
                vals = res.iter().map(|(k,v)| (k.to_owned(), serde_json::to_value(v).unwrap())).collect();
            },
            Some(submap) => {
                fiats.iter().for_each(|f| match submap.get(&f.ticker()) {
                    None => missing.push(f.clone()),
                    Some(rate) => {vals.insert(f.ticker(), serde_json::to_value(rate).unwrap());},
                })
            },
        }
        if missing.len() != 0 {
            let res: HashMap<String, f64> = client.multi_fiat_ticker(crypto, missing).await?;
            self.cached_tickers.insert(ct, res.clone());
            vals = res.iter().map(|(k,v)| (k.to_owned(), serde_json::to_value(v).unwrap())).collect();
        }
        serde_json::from_value(vals.into()).map_err(|e| e.into())
    }
}
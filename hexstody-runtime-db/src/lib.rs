use std::collections::HashMap;
use std::fmt::Debug;

use hexstody_api::domain::Symbol;
use hexstody_ticker_provider::client::TickerClient;
use hexstody_ticker_provider::client::Result as TickerResult;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::Map;

pub struct RuntimeState {
    /// Runtime cache of challenges to log in with a key
    pub challenges: HashMap<String, String>,
    /// Cached ticker info
    /// We store tikers refering by Symbol
    /// since we want to uniformely store both Crypto and Fiat tickers in the same map 
    pub cached_tickers: HashMap<Symbol, HashMap<Symbol, f64>>
}

impl RuntimeState {
    pub fn new() -> Self{
        RuntimeState{
            challenges: HashMap::new(),
            cached_tickers: HashMap::new()
        }
    }

    pub async fn symbol_to_symbol(&mut self, client: &TickerClient, from: Symbol, to: Symbol) -> TickerResult<f64>{
        let mrate = self.cached_tickers.get(&from).map(|sm| sm.get(&to)).flatten();
        match mrate {
            Some(rate) => Ok(rate.clone()),
            None => {
                let rate = client.symbol_to_symbol(&from, &to).await?;
                self.cached_tickers.entry(from).and_modify(|cm| {cm.insert(to, rate);});
                Ok(rate)
            },
        }
    }

    pub async fn symbol_to_symbols_generic<T>(&mut self, client: &TickerClient, from: Symbol, to: Vec<Symbol>) -> TickerResult<T>
    where T: DeserializeOwned + Debug 
    {
        let mut vals: Map<String, Value> = Map::new();
        let mut missing: Vec<Symbol> = Vec::new();
        let submap = self.cached_tickers.get(&from);
        match submap {
            None => {
                let res: HashMap<Symbol, f64> = client.symbol_to_symbols(&from, &to).await?;
                self.cached_tickers.insert(from.clone(), res.clone());
                vals = res.iter().map(|(k,v)| (k.symbol(), serde_json::to_value(v).unwrap())).collect();
            },
            Some(submap) => {
                to.iter().for_each(|t| match submap.get(&t) {
                    None => missing.push(t.clone()),
                    Some(rate) => {vals.insert(t.symbol(), serde_json::to_value(rate).unwrap());},
                })
            },
        }
        if missing.len() != 0 {
            let res: HashMap<Symbol, f64> = client.symbol_to_symbols(&from, &missing).await?;
            self.cached_tickers.insert(from, res.clone());
            vals = res.iter().map(|(k,v)| (k.symbol(), serde_json::to_value(v).unwrap())).collect();
        }
        serde_json::from_value(vals.into()).map_err(|e| e.into())
    }

    pub async fn symbol_to_symbols(&mut self, client: &TickerClient, from: Symbol, to: Vec<Symbol>) -> TickerResult<HashMap<Symbol, f64>>{
        let mut result: HashMap<Symbol, f64> = HashMap::new();
        let mut missing: Vec<Symbol> = Vec::new();
        let submap = self.cached_tickers.get(&from);
        match submap {
            None => {
                let res = client.symbol_to_symbols(&from, &to).await?;
                self.cached_tickers.insert(from.clone(), res.clone());
                return Ok(res) ;
            },
            Some(submap) => {
                to.iter().for_each(|t| match submap.get(&t) {
                    None => missing.push(t.clone()),
                    Some(rate) => {result.insert(t.clone(), rate.clone());},
                })
            },
        }
        if missing.len() != 0 {
            let res = client.symbol_to_symbols(&from, &missing).await?;
            self.cached_tickers.insert(from, res.clone());
            res.into_iter().for_each(|(k,v)| {
                result.insert(k, v);
            });
        }
        Ok(result)
    }

    pub fn tracked_pairs(&self) -> HashMap<Symbol, Vec<Symbol>>{
        self
            .cached_tickers
            .iter()
            .map(|(k,v)| (k.clone(), v.keys().cloned().collect()))
            .collect()
    }
}
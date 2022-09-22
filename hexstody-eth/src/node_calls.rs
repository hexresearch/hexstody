use crate::types::*;
use crate::conf::{NodeConfig, load_config};

pub fn estimate_eth_tx_gas(cfgp: &NodeConfig,
                    tx: &EthTransaction) -> Result<String, ureq::Error> {
    let cfg = cfgp.clone();
    let resp = ureq::get(&cfg.naddr())
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "method": "eth_estimateGas",
          "params": [tx],
          "id":"1"
        }))?;
    let resp_str = resp.into_string().unwrap();
    log::warn!("=======================================");
    log::warn!("Node Call: {:?}",resp_str);
    let resp_g : GethResponceOpt = (serde_json::from_str(&resp_str)).unwrap();
    log::warn!("Node Call Parsed: {:?}",resp_g);
    log::warn!("=======================================");
    return Ok(resp_str)
}

pub fn send_eth_tx(cfgp: &NodeConfig,
                    tx: &EthTransactionNoData) -> Result<String, ureq::Error> {
    let cfg = cfgp.clone();

    let resp = ureq::post(&cfg.naddr())
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "method": "eth_sendTransaction",
          "params": [tx],
          "id":"1"
        }))?;
    let resp_str = resp.into_string().unwrap();
    log::warn!("=======================================");
    log::warn!("Node Call: {:?}",tx);
    log::warn!("Node Resp: {:?}",resp_str);
    let resp_g : GethResponceOpt = (serde_json::from_str(&resp_str)).unwrap();
    log::warn!("Node Resp Parsed: {:?}",resp_g);
    log::warn!("=======================================");
    return Ok(resp_str)
}

pub fn send_erc20_tx(cfgp: &NodeConfig,
                    tx: &EthTransactionDefaultGas) -> Result<String, ureq::Error> {
    let cfg = cfgp.clone();
    log::warn!("Node Call: {:?}",tx);
    let resp = ureq::post(&cfg.naddr())
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "method": "eth_sendTransaction",
          "params": [tx],
          "id":"1"
        }))?;
    log::warn!("================Result=================");
    let resp_str = resp.into_string().unwrap();
    log::warn!("=======================================");
    log::warn!("Node Call: {:?}",tx);
    log::warn!("Node Resp: {:?}",resp_str);
    let resp_g : GethResponceOpt = (serde_json::from_str(&resp_str)).unwrap();
    log::warn!("Node Resp Parsed: {:?}",resp_g);
    log::warn!("=======================================");
    return Ok(resp_str)
}


pub fn balance_erc20_token(cfgp: &NodeConfig,
                    eth_call: &EthCall) -> Result<GethResponceOpt, ureq::Error> {
    let cfg = cfgp.clone();
    let req_json = ureq::json!({
          "jsonrpc": "2.0",
          "method": "eth_call",
          "params": [eth_call, "latest"],
          "id":"1"
        });
    log::warn!("=======================================");
    log::warn!("Node Call: {:?}",eth_call);
    log::warn!("Node req JSON: {:?}",req_json);
    let resp = ureq::post(&cfg.naddr())
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "method": "eth_call",
          "params": [eth_call, "latest"],
          "id":"1"
        }))?;
    let resp_str = resp.into_string().unwrap();
    log::warn!("Node Resp: {:?}",resp_str);
    let resp_g : GethResponceOpt = (serde_json::from_str(&resp_str)).unwrap();
    log::warn!("Node Resp Parsed: {:?}",resp_g);
    log::warn!("=======================================");
    return Ok(resp_g)
}

pub fn unlock_account(cfgp: &NodeConfig, ap: &AccountPersonal) -> Result<String, ureq::Error> {
    let cfg = cfgp.clone();
    let resp = ureq::post(&cfg.naddr())
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "method": "personal_unlockAccount",
          "params": [ap.address, ap.passphrase, ap.duration],
          "id":"1"
        }))?;
    let resp_str = resp.into_string().unwrap();
    log::warn!("=======================================");
    log::warn!("Node Call: {:?}",resp_str);
    let resp_g : GethResponceOpt = (serde_json::from_str(&resp_str)).unwrap();
    log::warn!("Node Call Parsed: {:?}",resp_g);
    log::warn!("=======================================");
    return Ok(resp_str)
}


pub fn balance_eth(cfgp: &NodeConfig, addr: &str) -> Result<GethResponceOpt, ureq::Error> {
    let cfg = cfgp.clone();
    let resp = ureq::post(&cfg.naddr())
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "method": "eth_getBalance",
          "params": [addr, "latest"],
          "id":"1"
        }))?;
    let resp_str = resp.into_string().unwrap();
    log::warn!("=======================================");
    log::warn!("Node Call: {:?}",resp_str);
    let resp_g : GethResponceOpt = (serde_json::from_str(&resp_str)).unwrap();
    log::warn!("Node Call Parsed: {:?}",resp_g);
    log::warn!("=======================================");
    return Ok(resp_g)
}

pub async fn get_balance_eth(acc: &str) -> Result<String, ureq::Error> {
    let cfg = load_config("config.json");
    let balance_resp_str = reqwest::get(
                        &("https://".to_owned() + &cfg.etherscan_api_prefix+".etherscan.io/api?module=account&action=balance&address="
                                     +acc+
                                     "&tag=latest&apikey="+&cfg.etherscan_api_key)
                                ).await
                                .unwrap()
                                .text()
                                .await
                                .unwrap();
    let bal_resp : BalanceResponce = (serde_json::from_str(&balance_resp_str)).unwrap();
    Ok(bal_resp.result)
}

pub async fn get_balance_token(acc: &str, token: &str, token_name: &str) -> Result<Erc20TokenBalance, ureq::Error> {
    let cfg = load_config("config.json");
    let resurl = "https://".to_owned() + &cfg.etherscan_api_prefix+".etherscan.io/api?module=account&action=tokenbalance&address="
                 +acc
                 +"&contractaddress="
                 +token
                 +"&tag=latest&apikey="+&cfg.etherscan_api_key;

    let balance_resp_str =
                    reqwest::get(resurl).await
                                .unwrap()
                                .text()
                                .await
                                .unwrap();
    let bal_resp : BalanceResponce = (serde_json::from_str(&balance_resp_str)).unwrap();
    let balance_token : Erc20TokenBalance = Erc20TokenBalance{
        tokenName:      token_name.to_string(),
        tokenBalance:    bal_resp.result
    };
    Ok(balance_token)
}

pub async fn get_history_eth(acc: &str) -> Result<Vec<Erc20HistUnitU>, ureq::Error> {
    let cfg = load_config("config.json");
    let resurl = "https://".to_owned() + &cfg.etherscan_api_prefix+".etherscan.io/api?module=account&action=txlist&address=" +
                 acc +
                 "&startblock=0&endblock=99999999&page=1&offset=20&sort=desc&apikey="+&cfg.etherscan_api_key;

    let eth_hist_str = reqwest::get(resurl).await
                                            .unwrap()
                                            .text()
                                            .await
                                            .unwrap();

    let eth_hist_pred : EthHistResp = (serde_json::from_str(&eth_hist_str)).unwrap();
    let hist_list : Vec<EthHistUnit> = eth_hist_pred.result;

    let hist_list_u : Vec<Erc20HistUnitU> = hist_list.iter()
                                                               .map(|x| {
                                                                   return Erc20HistUnitU {
                                                                       blockNumber : x.blockNumber.clone(),
                                                                       timeStamp : x.timeStamp.clone(),
                                                                       hash : x.hash.clone(),
                                                                       from : x.from.clone(),
                                                                       to : x.to.clone(),
                                                                       value : x.value.clone(),
                                                                       tokenName : "ETH".to_string(),
                                                                       gas: x.gas.clone(),
                                                                       gasPrice: x.gasPrice.clone(),
                                                                       contractAddress : x.contractAddress.clone(),
                                                                       confirmations : x.confirmations.clone(),
                                                                       addr : acc.to_string()
                                                                    };
                                                                }).collect();

    Ok(hist_list_u)
}

pub async fn get_history_token(acc: &str, token: &str, ticker: &str) -> Result<Erc20TokenHistory, ureq::Error> {
    let cfg = load_config("config.json");
    let resurl = "https://".to_owned() + &cfg.etherscan_api_prefix+".etherscan.io/api?module=account&action=tokentx&address=" +
                             acc +
                             "&contractaddress=" +
                             token +
                             "&startblock=0&endblock=99999999&page=1&offset=20&sort=desc&apikey="+&cfg.etherscan_api_key;

    let eth_hist_str = reqwest::get(resurl).await
                                            .unwrap()
                                            .text()
                                            .await
                                            .unwrap();

    let eth_hist_pred : Erc20HistResp = (serde_json::from_str(&eth_hist_str)).unwrap();
    let hist_list : Vec<Erc20HistUnit> = eth_hist_pred.result;

    let hist_list_u : Vec<Erc20HistUnitU> = hist_list.iter()
                                                               .map(|x| {
                                                                   return Erc20HistUnitU {
                                                                       blockNumber : x.blockNumber.clone(),
                                                                       timeStamp : x.timeStamp.clone(),
                                                                       hash : x.hash.clone(),
                                                                       from : x.from.clone(),
                                                                       to : x.to.clone(),
                                                                       value : x.value.clone(),
                                                                       tokenName : x.tokenSymbol.clone(),
                                                                       gas: x.gas.clone(),
                                                                       gasPrice: x.gasPrice.clone(),
                                                                       contractAddress : x.contractAddress.clone(),
                                                                       confirmations : x.confirmations.clone(),
                                                                       addr : acc.to_string()
                                                                    };
                                                                }).collect();
    let tkn = Erc20Token{ticker: ticker.to_string(), name: ticker.to_string(), contract: token.to_string()};
    let hist = Erc20TokenHistory{token: tkn, history: hist_list_u};
    Ok(hist)
}

pub fn get_node_version(cfgp: &NodeConfig) -> Result<String, ureq::Error> {
    let cfg = cfgp.clone();
    let body: GethResponce = ureq::post(&cfg.naddr())
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "method": "web3_clientVersion",
          "params": [],
          "id":"1"
        }))?
        .into_json()?;
    return Ok(body.result)
}

pub fn create_new_account(cfgp: &NodeConfig, login: &str) -> Result<String, ureq::Error> {
    let cfg = cfgp.clone();
    let body: GethResponce = ureq::post(&cfg.naddr())
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "method": "personal_newAccount",
          "params": [login],
          "id":"1"
        }))?
        .into_json()?;
    return Ok(body.result)
}

#[allow(dead_code)]
pub fn get_account_list(cfgp: &NodeConfig) -> Result<Vec<String>, ureq::Error> {
    let cfg = cfgp.clone();
    let body: GethLResponce = ureq::post(&cfg.naddr())
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "method": "personal_listAccounts",
          "params": [],
          "id":"1"
        }))?
        .into_json()?;
    return Ok(body.result)
}

#[allow(dead_code)]
pub fn get_eth_balance(cfgp: &NodeConfig, acc: &str) -> Result<f64, ureq::Error> {
    let cfg = cfgp.clone();
    let body: GethResponce = ureq::post(&cfg.naddr())
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "method": "eth_getBalance",
          "params": [acc,"latest"],
          "id":"1"
        }))?
        .into_json()?;
    let mres_u64 = eth_to_decimal(&body.result);
    println!("!!Debug!!");
    println!("mres: {:?}",mres_u64);
    match mres_u64 {
        Ok(res_u64) => {return Ok((res_u64 as f64)/1000000000000000000.0)}
        Err(_) => {return Ok(0 as f64)}
    };
}

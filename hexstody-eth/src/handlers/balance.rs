use crate::types::*;
use crate::utils::*;
use crate::node_calls;
use crate::db_functions;
use crate::conf::{NodeConfig, load_config};

use rocket::{get,post,State};
use rocket::http::{Status, ContentType};
use rocket::serde::json::Json;
use rocket_db_pools::{Database, Connection};
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};

#[openapi(tag = "balance")]
#[get("/balance/eth/total")]
pub async fn balance_eth_total(db: Connection<MyDb>) -> (Status, (ContentType, String)) {
    let total_balances_prep = db_functions::pg_query_total_eth(db).await.unwrap();
    let mut eth_total_balance :u64 = 0;
    for bal in total_balances_prep{
        eth_total_balance = eth_total_balance + bal.parse::<u64>().unwrap();
    };
    let eth_total = EthBalanceU64{balance: eth_total_balance};
    let json_res = serde_json::to_string(&eth_total);
    return (Status::Ok, (ContentType::JSON, json_res.unwrap()))
}

#[openapi(tag = "balance")]
#[get("/balance/erc20/total")]
pub async fn balance_erc20_total(db: Connection<MyDb>) -> (Status, (ContentType, String)) {
    let total_balances_prep = db_functions::pg_query_total_tokens(db).await.unwrap();
    let mut total = TotalBalanceErc20 {balance : [].to_vec()};
    for bal in total_balances_prep{
        for tkn in bal{
            let mut exists : bool = false;
            for t in &mut total.balance{
                if tkn.tokenName == t.tokenName{
                    t.tokenBalance += tkn.tokenBalance.parse::<u64>().unwrap();
                    exists = true;
                }
            }
            if !exists {
                let ntkn = Erc20TokenBalanceU64{tokenName: tkn.tokenName, tokenBalance: tkn.tokenBalance.parse::<u64>().unwrap()};
                let _ = &total.balance.push(ntkn);
            };
        }
    }
    let json_res = serde_json::to_string(&total);
    return (Status::Ok, (ContentType::JSON, json_res.unwrap()))
}

#[openapi(tag = "balance")]
#[get("/balance/eth/login/<login>")]
pub async fn balance_eth_login(login: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  let user = db_functions::pg_query_user(db,login).await.unwrap();
  let user_address = &user.address;
  log::warn!("=======================<BALANCE ETH>==========================");
  log::warn!("Username {:?}",login);
  log::warn!("Address {:?}",user_address);
  let r_res = node_calls::balance_eth(cfg,user_address);
  match r_res{
      Err(e) => {
          log::warn!("NodeErr Response {:?}",&e);
          return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
      },
      Ok(res) => {
          match &res.result{
              None => {
                  log::warn!("No result error");
                  return (Status::InternalServerError,(ContentType::JSON, "No result error".to_string()));
              }
              Some(res_str) => {
                  let json_res = serde_json::to_string(&hxt_str_to_f64(res_str));
                  log::warn!("=======================</BALANCE ETH>==========================");
                  return (Status::Ok, (ContentType::JSON, format!("{}",&hxt_str_to_f64(res_str))));
              }
          }
      }
  }
}

#[openapi(tag = "balance")]
#[get("/balance/eth/address/<address>")]
pub async fn balance_eth_address(address: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  log::warn!("=======================<BALANCE ETH>==========================");
  log::warn!("address {:?}",address);
  let r_res = node_calls::balance_eth(cfg,address);
  match r_res{
      Err(e) => {
          log::warn!("NodeErr Response {:?}",&e);
          return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
      },
      Ok(res) => {
          match &res.result{
              None => {
                  log::warn!("No result error");
                  return (Status::InternalServerError,(ContentType::JSON, "No result error".to_string()));
              }
              Some(res_str) => {
                  log::warn!("=======================</BALANCE ETH>==========================");
                  return (Status::Ok, (ContentType::JSON, format!("{}",&hxt_str_to_f64(res_str))));
              }
          }
      }
  }
}

#[openapi(tag = "balance")]
#[get("/balance/erc20/login/<login>/<token_address>")]
pub async fn balance_erc20_login(login: &str,
                  token_address: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  let user = db_functions::pg_query_user(db,login).await.unwrap();
  let user_address = &user.address;
  log::warn!("=======================<BALANCE ERC20>==========================");
  log::warn!("Username {:?}",login);
  log::warn!("Address {:?}",user_address);
  log::warn!("TokenAddress {:?}",token_address);
  let eth_call = EthCall {
      to : token_address.to_string(),
      data: "0x70a08231000000000000000000000000".to_string() + &user_address[2..user_address.len()]
  };
  log::warn!("eth_call: {:?}", eth_call);
  let r_res = node_calls::balance_erc20_token(cfg,&eth_call);
  log::warn!("Call result {:?}",r_res);
  match r_res{
      Err(e) => {
          log::warn!("NodeErr Response {:?}",&e);
          return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
      },
      Ok(res) => {
          match &res.result{
              None => {
                  log::warn!("No result error");
                  return (Status::InternalServerError,(ContentType::JSON, "No result error".to_string()));
              }
              Some(res_str) => {
                  let json_res = serde_json::to_string(&hxt_str_to_f64(res_str));
                  log::warn!("=======================</BALANCE ERC20>==========================");
                  return (Status::Ok, (ContentType::JSON, format!("{}",&hxt_str_to_f64(res_str))));
              }
          }
      }
  }
}

#[openapi(tag = "balance")]
#[get("/balance/erc20/address/<address>/<token_address>")]
pub async fn balance_erc20_address(address: &str,
                  token_address: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  log::warn!("=======================<BALANCE ERC20>==========================");
  log::warn!("Address {:?}",address);
  log::warn!("TokenAddress {:?}",token_address);
  let eth_call = EthCall {
      to : token_address.to_string(),
      data: "0x70a08231000000000000000000000000".to_string() + &address[2..address.len()]
  };
  log::warn!("eth_call: {:?}", eth_call);
  let r_res = node_calls::balance_erc20_token(cfg,&eth_call);
  log::warn!("Call result {:?}",r_res);
  match r_res{
      Err(e) => {
          log::warn!("NodeErr Response {:?}",&e);
          return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
      },
      Ok(res) => {
          match &res.result{
              None => {
                  log::warn!("No result error");
                  return (Status::InternalServerError,(ContentType::JSON, "No result error".to_string()));
              }
              Some(res_str) => {
                  let json_res = serde_json::to_string(&hxt_str_to_f64(res_str));
                  log::warn!("=======================</BALANCE ERC20>==========================");
                  return (Status::Ok, (ContentType::JSON, format!("{}",&hxt_str_to_f64(res_str))));
              }
          }
      }
  }
}

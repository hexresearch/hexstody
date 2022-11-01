use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::i64;
use sqlx::postgres::{Postgres};
use rocket_db_pools::{sqlx, Database};

pub type Pool = sqlx::Pool<Postgres>;

#[derive(Database)]
#[database("hexstody")]
pub struct MyDb(sqlx::PgPool);



#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponce{
  pub status  : String
 ,pub message : String
 ,pub result  : String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EthTransaction{
  pub from     : String
 ,pub to       : String
 ,pub value    : String
 ,pub gas      : String
 ,pub gasPrice : String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EthCall{
  pub to   : String
 ,pub data : String
 }

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EthTransactionDefaultGas{
  pub from  : String
 ,pub to    : String
 ,pub value : String
 ,pub data  : String
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EthTransactionNoData{
  pub from  : String
 ,pub to    : String
 ,pub value : String
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct AccountPersonal{
  pub address    : String
 ,pub passphrase : String
 ,pub duration   : i32
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserEth{
  pub login   : String
 ,pub address : String
 ,pub data    : Option<serde_json::Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GethRequest{
  jsonrpc : String
 ,method  : String
 ,params  : Vec<String>
 ,id      : u8
}

#[derive(Debug, Deserialize)]
pub struct GethResponce{
  pub jsonrpc : String
 ,pub id      : String
 ,pub result  : String
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct GethResponceOpt{
  pub jsonrpc : String
 ,pub id      : String
 ,pub error   : Option<NodeError>
 ,pub result  : Option<String>
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct NodeError{
  pub code    : i32
 ,pub message : String
}

#[derive(Deserialize)]
pub struct GethLResponce{
  pub jsonrpc : String
 ,pub id      : String
 ,pub result  : Vec<String>
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct SendTxData{
  pub to    : String
 ,pub val   : String
 ,pub hash  : String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account{
  address : String
 ,balance : u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub login: String,
    pub address: String,
}
// "{\"status\":\"1\",\"message\":\"OK\",\"result\":\"343270355903185816963191\"}"
#[derive(Debug, Serialize, Deserialize)]
pub struct BalResp {
    pub status: i32,
    pub message: String,
    pub bal: i64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub bal: u64
}


#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EthHistResp {
    pub status: String,
    pub message: String,
    pub result: Vec<EthHistUnit>
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Erc20HistResp {
    pub status: String,
    pub message: String,
    pub result: Vec<Erc20HistUnit>
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EthHistUnit {
    pub blockNumber: String,
    pub timeStamp: String,
    pub hash: String,
    pub nonce: String,
    pub blockHash: String,
    pub transactionIndex: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: String,
    pub gasPrice: String,
    pub isError: String,
    pub txreceipt_status: String,
    pub input: String,
    pub contractAddress: String,
    pub cumulativeGasUsed: String,
    pub gasUsed: String,
    pub confirmations: String,
    pub methodId: String,
    pub functionName: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EthHistUnitU {
    pub blockNumber: String,
    pub timeStamp: String,
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: String,
    pub gasPrice: String,
    pub contractAddress: String,
    pub confirmations: String,
    pub addr: String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Erc20HistUnit {
    pub blockNumber: String,
    pub timeStamp: String,
    pub hash: String,
    pub nonce: String,
    pub blockHash: String,
    pub from: String,
    pub contractAddress: String,
    pub to: String,
    pub value: String,
    pub tokenName: String,
    pub tokenSymbol: String,
    pub tokenDecimal: String,
    pub transactionIndex: String,
    pub gas: String,
    pub gasPrice: String,
    pub gasUsed: String,
    pub cumulativeGasUsed: String,
    pub input: String,
    pub confirmations: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Erc20HistUnitU {
    pub blockNumber: String,
    pub timeStamp: String,
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub tokenName: String,
    pub gas: String,
    pub gasPrice: String,
    pub contractAddress: String,
    pub confirmations: String,
    pub addr: String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct UserData{
    pub tokens: Vec<Erc20Token>,
    pub historyEth: Vec<Erc20HistUnitU>,
    pub historyTokens: Vec<Erc20TokenHistory>,
    pub balanceEth: String,
    pub balanceTokens: Vec<Erc20TokenBalance>
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct TotalBalanceErc20{
    pub balance: Vec<Erc20TokenBalanceU64>
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Erc20TokenOld{
    pub tokenName: String,
    pub tokenAddr: String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Erc20Token {
    pub ticker: String,
    pub name: String,
    pub contract: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Erc20TokenHistory {
    pub token: Erc20Token,
    pub history: Vec<Erc20HistUnitU>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Erc20TokenWrapper{
    pub tokens: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EthBalanceWrapper{
    pub balance   : Option<serde_json::Value>
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Erc20TokenBalanceWrapper{
    pub erc20_balance   : Option<serde_json::Value>
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Erc20TokenBalance{
    pub tokenName: String,
    pub tokenBalance: String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Erc20TokenBalanceU64{
    pub tokenName: String,
    pub tokenBalance: u64
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EthBalanceU64{
    pub balance: u64
}

#[allow(dead_code)]
impl Account{
  pub fn bal(self) -> u64 {
    let decimal_bal = self.balance;
    return decimal_bal;
  }
}

#[allow(dead_code)]
pub fn eth_to_decimal(s : &String) -> Result<u64, std::num::ParseIntError> {
  let without_prefix = s.trim_start_matches("0x");
  let z = u64::from_str_radix(without_prefix, 16);
  return z;
}

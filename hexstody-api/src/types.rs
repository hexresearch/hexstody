use base64;
use chrono::NaiveDateTime;
use hexstody_btc_api::bitcoin::txid::BtcTxid;
use okapi::openapi3::*;
use p256::{ecdsa::Signature, pkcs8::DecodePublicKey, PublicKey};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    serde::json::json,
};
use rocket_okapi::{
    gen::OpenApiGenerator,
    okapi::schemars::{self, JsonSchema},
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::CurrencyTxId;

use super::domain::currency::{BtcAddress, Currency, CurrencyAddress, Erc20Token};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TickerETH {
    pub USD: f32,
    pub RUB: f32
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EthHistResp {
    pub status: String,
    pub message: String,
    pub result: Vec<EthHistUnit>
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Erc20HistResp {
    pub status: String,
    pub message: String,
    pub result: Vec<Erc20HistUnit>
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UserEth{
  pub login   : String
 ,pub address : String
 ,pub data    : UserData
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UserData{
    pub tokens: Vec<Erc20Token>,
    pub historyEth: Vec<Erc20HistUnitU>,
    pub historyTokens: Vec<Erc20TokenHistory>,
    pub balanceEth: String,
    pub balanceTokens: Vec<Erc20TokenBalance>
}


#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Erc20TokenBalance{
    pub tokenName: String,
    pub tokenBalance: String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Erc20TokenHistory {
    pub token: Erc20Token,
    pub history: Vec<Erc20HistUnitU>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EthFeeResp {
    pub status: String,
    pub message: String,
    pub result: EthGasPrice
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EthGasPrice {
    pub LastBlock: String,
    pub SafeGasPrice: String,
    pub ProposeGasPrice: String,
    pub FastGasPrice: String,
    pub suggestBaseFee: String,
    pub gasUsedRatio: String
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceItem {
    pub currency: Currency,
    pub value: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositHistoryItem {
    pub currency: Currency,
    pub date: NaiveDateTime,
    pub value: u64,
    pub number_of_confirmations: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct WithdrawalHistoryItem {
    pub currency: Currency,
    pub date: NaiveDateTime,
    pub value: u64,
    pub status: WithdrawalRequestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum HistoryItem {
    Deposit(DepositHistoryItem),
    Withdrawal(WithdrawalHistoryItem),
}

pub fn history_item_time(h: &HistoryItem) -> &NaiveDateTime {
    match h {
        HistoryItem::Deposit(d) => &d.date,
        HistoryItem::Withdrawal(w) => &w.date,
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Balance {
    pub balances: Vec<BalanceItem>,
}

impl Balance {
    pub fn by_currency(&self, curr: &Currency) -> Option<&BalanceItem> {
        self.balances.iter().find(|i| i.currency == *curr)
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct History {
    pub target_number_of_confirmations: i16,
    pub history_items: Vec<HistoryItem>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SignupEmail {
    /// Unique email
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SigninEmail {
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WithdrawalRequestInfo {
    /// User which initiated withdrawal request
    #[schemars(example = "example_user")]
    pub user: String,
    /// Receiving address
    #[schemars(example = "example_address")]
    pub address: CurrencyAddress,
    /// Amount of tokens to transfer
    #[schemars(example = "example_amount")]
    pub amount: u64,
}

/// Auxiliary data type to display `WithdrawalRequest` on the page
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WithdrawalRequest {
    /// Request ID
    #[schemars(example = "example_uuid")]
    pub id: Uuid,
    /// User which initiated request
    #[schemars(example = "example_user")]
    pub user: String,
    /// Receiving address
    #[schemars(example = "example_address")]
    pub address: CurrencyAddress,
    /// When the request was created
    #[schemars(example = "example_datetime")]
    pub created_at: String,
    /// Amount of tokens to transfer
    #[schemars(example = "example_amount")]
    pub amount: u64,
    /// Some request require manual confirmation
    #[schemars(example = "example_confirmation_status")]
    pub confirmation_status: WithdrawalRequestStatus,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UserWithdrawRequest {
    pub address: CurrencyAddress,
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ConfirmationData {
    /// Withdrawal request ID
    #[schemars(example = "example_uuid")]
    pub id: Uuid,
    /// User which initiated withdrawal request
    #[schemars(example = "example_user")]
    pub user: String,
    /// Receiving address
    #[schemars(example = "example_address")]
    pub address: CurrencyAddress,
    /// When the request was created
    #[schemars(example = "example_datetime")]
    pub created_at: String,
    /// Amount of tokens to transfer
    #[schemars(example = "example_amount")]
    pub amount: u64,
}

fn example_uuid() -> &'static str {
    "fdb12d51-0e3f-4ff8-821e-fbc255d8e413"
}

fn example_user() -> &'static str {
    "Alice"
}

fn example_address() -> CurrencyAddress {
    CurrencyAddress::BTC(BtcAddress {
        addr: "1BNwxHGaFbeUBitpjy2AsKpJ29Ybxntqvb".to_owned(),
    })
}

fn example_datetime() -> &'static str {
    "2012-04-23T18:25:43.511Z"
}

fn example_amount() -> u64 {
    3
}

fn example_confirmation_status() -> WithdrawalRequestStatus {
    WithdrawalRequestStatus::InProgress { confirmations: 1 }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum WithdrawalRequestStatus {
    /// Number of confirmations received
    InProgress {
        confirmations: i16,
    },
    /// Confirmed by operators, but not yet sent to the node
    Confirmed,
    /// Tx sent to the node
    Completed {
        /// Time when the request was processed
        confirmed_at: NaiveDateTime,
        /// Txid
        txid: CurrencyTxId,
        /// Fee paid is sats. If an error occured, fee=0
        fee: u64
    },
    /// Rejected by operators
    OpRejected,
    /// Rejected by the node
    NodeRejected {
        /// Node
        reason: String
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositInfo {
    pub address: String,
}

/// Signature data that comes from operators
/// when they sign or reject requests.
/// This data type is used for authorization.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SignatureData {
    pub signature: Signature,
    pub nonce: u64,
    pub public_key: PublicKey,
}

// Dummy JsonSchema. Needed for derive JsonSchema elsewhere
impl JsonSchema for SignatureData {
    fn schema_name() -> String {
        "Signature data".to_string()
    }
    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema::SchemaObject::default())
    }
}

#[derive(Debug)]
pub enum SignatureError {
    MissingSignatureData,
    InvalidSignatureDataLength,
    InvalidSignature,
    InvalidNonce,
    InvalidPublicKey,
    UnknownPublicKey,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SignatureData {
    type Error = SignatureError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Signature-Data") {
            None => {
                return Outcome::Failure((Status::BadRequest, SignatureError::MissingSignatureData))
            }
            Some(sig_data) => {
                let sig_data_vec: Vec<&str> = sig_data.split(':').collect();
                match sig_data_vec[..] {
                    [signature_str, nonce_str, public_key_str] => {
                        let signature = match base64::decode(signature_str) {
                            Ok(sig_der) => match Signature::from_der(&sig_der) {
                                Ok(sig) => sig,
                                Err(_) => {
                                    return Outcome::Failure((
                                        Status::BadRequest,
                                        SignatureError::InvalidSignature,
                                    ));
                                }
                            },
                            Err(_) => {
                                return Outcome::Failure((
                                    Status::BadRequest,
                                    SignatureError::InvalidSignature,
                                ));
                            }
                        };
                        let nonce = match nonce_str.parse::<u64>() {
                            Ok(n) => n,
                            Err(_) => {
                                return Outcome::Failure((
                                    Status::BadRequest,
                                    SignatureError::InvalidNonce,
                                ))
                            }
                        };
                        let public_key = match base64::decode(public_key_str) {
                            Ok(key_der) => match PublicKey::from_public_key_der(&key_der) {
                                Ok(key) => key,
                                Err(_) => {
                                    return Outcome::Failure((
                                        Status::BadRequest,
                                        SignatureError::InvalidPublicKey,
                                    ))
                                }
                            },
                            Err(_) => {
                                return Outcome::Failure((
                                    Status::BadRequest,
                                    SignatureError::InvalidPublicKey,
                                ))
                            }
                        };
                        return Outcome::Success(SignatureData {
                            signature: signature,
                            nonce: nonce,
                            public_key: public_key,
                        });
                    }
                    _ => {
                        return Outcome::Failure((
                            Status::BadRequest,
                            SignatureError::InvalidSignatureDataLength,
                        ))
                    }
                };
            }
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for SignatureData {
    fn from_request_input(
        gen: &mut OpenApiGenerator,
        _name: String,
        required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let schema = gen.json_schema::<String>();
        let description = Some(
            "Contains a string with a serialized digital signature,
            a nonce, and the corresponding public key.
            Format is: \"signature:nonce:public_key\".
            Where \"signature\" is in Base64 encoded DER format.
            \"nonce\" is an UTF-8 string containing 64-bit unsigned integer.
            \"public_key\" is in Base64 encoded DER format."
                .to_owned(),
        );
        let example = Some(json!("MEYCIQCIlvwe8VWpYMFR/0kEbIU+Wh8VU9V3NNxOxM6/obuY4gIhAMP9RzhIwIOekO2EAGONfn/jkERPXlM/U+k9q3uNyRTf:1654706913710:MDkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDIgADWlzihGEBq52xGU9C7rbuYs3hloPAmWPmCkf9XgqkBrY="));
        Ok(RequestHeaderInput::Parameter(Parameter {
            name: "Signature-Data".to_owned(),
            location: "header".to_owned(),
            description: description,
            required,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Schema {
                style: None,
                explode: None,
                allow_reserved: false,
                schema,
                example: example,
                examples: None,
            },
            extensions: Object::default(),
        }))
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FeeResponse {
    /// Estimate fee rate in BTC/kB.
    #[schemars(example = "example_fee")]
    pub fee_rate: u64,
    /// Block number where estimate was found. None means that there was an error and a default value was used
    #[schemars(example = "example_block_height")]
    pub block: Option<i64>,
}

fn example_fee() -> u64 {
    5
}

fn example_block_height() -> Option<i64> {
    Some(12345)
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HotBalanceResponse{
    /// Total balance of the hot wallet in sat
    pub balance: u64
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum WithdrawalRequestDecisionType {
    Confirm,
    Reject,
}

impl WithdrawalRequestDecisionType {
    pub fn to_json(&self) -> String {
        rocket::serde::json::to_string(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ConfirmedWithdrawal {
    /// Request ID
    pub id: Uuid,
    /// User which initiated request
    pub user: String,
    /// Receiving address
    pub address: CurrencyAddress,
    /// When the request was created
    pub created_at: String,
    /// Amount of tokens to transfer
    pub amount: u64,
    /// Confirmations received from operators
    pub confirmations: Vec<SignatureData>,
    /// Rejections received from operators
    pub rejections: Vec<SignatureData>,
}

#[derive(Debug)]
pub enum ConfirmedWithdrawalError {
    MissingId,
    InvalidId,
    MissingUser,
    MissingAddress,
    InvalidAddress,
    MissingCreatedAt,
    MissingAmount,
    InvalidAmount,
    InsufficientConfirmations,
    MoreRejections,
    InvalidSignature(SignatureError),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WithdrawalResponse {
    /// Request ID
    pub id: Uuid,
    /// Transaction ID
    pub txid: BtcTxid,
    /// Fee paid in satoshi
    pub fee: Option<u64>
}

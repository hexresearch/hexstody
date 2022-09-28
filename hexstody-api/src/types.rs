use std::fmt;

use base64;
use chrono::NaiveDateTime;
use hexstody_btc_api::bitcoin::txid::BtcTxid;
use okapi::openapi3::*;
use p256::{ecdsa::Signature, pkcs8::DecodePublicKey, PublicKey};
use rocket::{
    http::{
        uri::fmt::{Formatter, FromUriParam, Query, UriDisplay},
        Status,
    },
    request::{FromRequest, Outcome, Request},
    serde::json::json,
    FromFormField,
};
use rocket_okapi::{
    gen::OpenApiGenerator,
    okapi::schemars::{self, JsonSchema},
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{CurrencyTxId, Email, PhoneNumber, TgName};

use super::domain::currency::{BtcAddress, Currency, CurrencyAddress, Erc20Token};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TickerETH {
    pub USD: f32,
    pub RUB: f32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EthHistResp {
    pub status: String,
    pub message: String,
    pub result: Vec<EthHistUnit>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Erc20HistResp {
    pub status: String,
    pub message: String,
    pub result: Vec<Erc20HistUnit>,
}

#[allow(non_snake_case)]
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

#[allow(non_snake_case)]
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
    pub addr: String,
}

#[allow(non_snake_case)]
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
    pub addr: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UserEth {
    pub login: String,
    pub address: String,
    pub data: UserData,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UserData {
    pub tokens: Vec<Erc20Token>,
    pub historyEth: Vec<Erc20HistUnitU>,
    pub historyTokens: Vec<Erc20TokenHistory>,
    pub balanceEth: String,
    pub balanceTokens: Vec<Erc20TokenBalance>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<Email>,
    pub phone: Option<PhoneNumber>,
    pub tg_name: Option<TgName>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Erc20TokenBalance {
    pub tokenName: String,
    pub tokenBalance: String,
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
    pub result: EthGasPrice,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EthGasPrice {
    pub LastBlock: String,
    pub SafeGasPrice: String,
    pub ProposeGasPrice: String,
    pub FastGasPrice: String,
    pub suggestBaseFee: String,
    pub gasUsedRatio: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct BalanceItem {
    pub currency: Currency,
    pub value: u64,
    pub limit_info: LimitInfo,
}

impl PartialOrd for BalanceItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.currency.partial_cmp(&other.currency)
    }
}

impl Ord for BalanceItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.currency.cmp(&other.currency)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositHistoryItem {
    pub currency: Currency,
    pub date: NaiveDateTime,
    pub value: u64,
    pub number_of_confirmations: u64,
    pub txid: CurrencyTxId,
    pub to_address: CurrencyAddress,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct WithdrawalHistoryItem {
    pub currency: Currency,
    pub date: NaiveDateTime,
    pub value: u64,
    pub status: WithdrawalRequestStatus,
    //temp field to give txid for ETH and tokens while status not working
    pub txid: Option<CurrencyTxId>,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SignupEmail {
    /// Unique user name
    pub user: String,
    pub invite: Invite,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SigninEmail {
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PasswordChange {
    pub old_password: String,
    pub new_password: String,
}

/// Auxiliary data type to display `WithdrawalRequest` on the page
// NOTE: fields order must be the same as in 'ConfirmationData' struct
// otherwise signature verification will fail
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

// NOTE: fields order must be the same as in 'WithdrawalRequest' struct
// otherwise signature verification will fail
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
    InProgress { confirmations: i16 },
    /// Confirmed by operators, but not yet sent to the node
    Confirmed,
    /// Tx sent to the node
    Completed {
        /// Time when the request was processed
        confirmed_at: NaiveDateTime,
        /// Txid
        txid: CurrencyTxId,
        /// Fee paid is sats. If an error occured, fee is None
        fee: Option<u64>,
        /// Tx inputs
        input_addresses: Vec<CurrencyAddress>,
        /// Tx outputs
        output_addresses: Vec<CurrencyAddress>,
    },
    /// Rejected by operators
    OpRejected,
    /// Rejected by the node
    NodeRejected {
        /// Node
        reason: String,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, JsonSchema, FromFormField)]
pub enum WithdrawalFilter {
    All,
    Pending,
    Confirmed,
    Completed,
    OpRejected,
    NodeRejected
}

impl ToString for WithdrawalFilter {
    fn to_string(&self) -> String {
        match self {
            WithdrawalFilter::All => "all".to_owned(),
            WithdrawalFilter::Pending => "pending".to_owned(),
            WithdrawalFilter::Confirmed => "confirmed".to_owned(),
            WithdrawalFilter::Completed => "completed".to_owned(),
            WithdrawalFilter::OpRejected => "oprejected".to_owned(),
            WithdrawalFilter::NodeRejected => "noderejected".to_owned(),
        }
    }
}

impl UriDisplay<Query> for WithdrawalFilter {
    fn fmt(&self, f: &mut Formatter<Query>) -> fmt::Result {
        f.write_value(self.to_string().as_str())
    }
}

impl<'a> FromUriParam<Query, &WithdrawalFilter> for WithdrawalFilter {
    type Target = WithdrawalFilter;

    fn from_uri_param(filt: &WithdrawalFilter) -> WithdrawalFilter {
        filt.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositInfo {
    pub address: String,
    pub qr_code_base64: String,
    pub tab: String,
    pub currency: String,
}

/// Signature data that comes from operators
/// when they sign or reject requests.
/// This data type is used for authorization.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
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
pub struct HotBalanceResponse {
    /// Total balance of the hot wallet in sat
    pub balance: u64,
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
    pub fee: Option<u64>,
    /// Input addresses
    pub input_addresses: Vec<CurrencyAddress>,
    /// Output addresses
    pub output_addresses: Vec<CurrencyAddress>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetTokensResponse {
    pub tokens: Vec<TokenInfo>,
}

#[derive(Debug, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct TokenInfo {
    pub token: Erc20Token,
    pub balance: u64,
    pub finalized_balance: u64,
    pub is_active: bool,
}

impl PartialOrd for TokenInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.token.partial_cmp(&other.token)
    }
}

impl Ord for TokenInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.token.cmp(&other.token)
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TokenActionRequest {
    pub token: Erc20Token,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, JsonSchema,
)]
pub struct Invite {
    pub invite: Uuid,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InviteRequest {
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InviteResp {
    pub invite: Invite,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EthHotWalletBalanceResponse {
    pub balance: u64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Erc20Balance {
    pub token_name: String,
    pub token_balance: u64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Erc20HotWalletBalanceResponse {
    pub balance: Vec<Erc20Balance>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone, JsonSchema)]
pub enum LimitSpan {
    Day,
    Week,
    Month,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Limit {
    pub amount: u64,
    pub span: LimitSpan,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct LimitChangeReq {
    pub currency: Currency,
    pub limit: Limit,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum LimitChangeStatus {
    InProgress { confirmations: i16, rejections: i16 },
    Completed,
    Rejected,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy, JsonSchema, FromFormField)]
pub enum LimitChangeFilter {
    All,
    Pending,
    Completed,
    Rejected
}

impl ToString for LimitChangeFilter {
    fn to_string(&self) -> String {
        match self {
            LimitChangeFilter::All => "all".to_owned(),
            LimitChangeFilter::Completed => "completed".to_owned(),
            LimitChangeFilter::Rejected => "rejected".to_owned(),
            LimitChangeFilter::Pending => "pending".to_owned()
        }
    }
}

impl UriDisplay<Query> for LimitChangeFilter {
    fn fmt(&self, f: &mut Formatter<Query>) -> fmt::Result {
        f.write_value(self.to_string().as_str())
    }
}

impl<'a> FromUriParam<Query, &LimitChangeFilter> for LimitChangeFilter {
    type Target = LimitChangeFilter;

    fn from_uri_param(filt: &LimitChangeFilter) -> LimitChangeFilter {
        filt.clone()
    }
}


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct LimitChangeResponse {
    pub id: Uuid,
    pub user: String,
    pub created_at: String,
    pub currency: Currency,
    pub limit: Limit,
    pub status: LimitChangeStatus,
}

// NOTE: fields order must be the same as in 'LimitConfirmationData' struct
// otherwise signature verification will fail
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct LimitChangeOpResponse {
    pub id: Uuid,
    pub user: String,
    pub created_at: String,
    pub currency: Currency,
    pub current_limit: Limit,
    pub requested_limit: Limit,
    pub status: LimitChangeStatus,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq, JsonSchema)]
pub struct LimitInfo {
    pub limit: Limit,
    pub spent: u64,
}

impl Default for LimitInfo {
    fn default() -> Self {
        Self {
            limit: Limit {
                amount: 0,
                span: LimitSpan::Day,
            },
            spent: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct LimitApiResp {
    pub currency: Currency,
    pub limit_info: LimitInfo,
}

impl PartialOrd for LimitApiResp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.currency.partial_cmp(&other.currency)
    }
}

impl Ord for LimitApiResp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.currency.cmp(&other.currency)
    }
}

// NOTE: fields order must be the same as in 'LimitChangeOpResponse' struct
// otherwise signature verification will fail
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct LimitConfirmationData {
    pub id: Uuid,
    pub user: String,
    pub created_at: String,
    pub currency: Currency,
    pub requested_limit: Limit,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum LimitChangeDecisionType {
    Confirm,
    Reject,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ConfigChangeRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tg_name: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ExchangeRequest {
    pub currency_from: Currency,
    pub currency_to: Currency,
    pub amount_from: u64,
    pub amount_to: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, JsonSchema)]
#[serde(tag = "type")]
pub enum ExchangeStatus {
    Completed,
    Rejected,
    InProgress { confirmations: i16, rejections: i16 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ExchangeOrder {
    pub user: String,
    pub id: Uuid,
    pub currency_from: Currency,
    pub currency_to: Currency,
    pub amount_from: u64,
    pub amount_to: u64,
    pub created_at: String,
    pub status: ExchangeStatus,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ExchangeConfirmationData {
    pub user: String,
    pub id: Uuid,
    pub currency_from: Currency,
    pub currency_to: Currency,
    pub amount_from: u64,
    pub amount_to: u64,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema, FromFormField, Copy)]
pub enum ExchangeFilter {
    All,
    Pending,
    Completed,
    Rejected,
}

impl UriDisplay<Query> for ExchangeFilter {
    fn fmt(&self, f: &mut Formatter<Query>) -> fmt::Result {
        f.write_value(self.to_string().as_str())
    }
}

impl ToString for ExchangeFilter {
    fn to_string(&self) -> String {
        match self {
            ExchangeFilter::All => "all".to_owned(),
            ExchangeFilter::Completed => "completed".to_owned(),
            ExchangeFilter::Rejected => "rejected".to_owned(),
            ExchangeFilter::Pending => "pending".to_owned(),
        }
    }
}

impl<'a> FromUriParam<Query, &str> for ExchangeFilter {
    type Target = ExchangeFilter;

    fn from_uri_param(filt: &str) -> ExchangeFilter {
        match filt.to_lowercase().as_str() {
            "all" => ExchangeFilter::All,
            "completed" => ExchangeFilter::Completed,
            "rejected" => ExchangeFilter::Rejected,
            "pending" => ExchangeFilter::Pending,
            _ => ExchangeFilter::All,
        }
    }
}

impl<'a> FromUriParam<Query, &ExchangeFilter> for ExchangeFilter {
    type Target = ExchangeFilter;

    fn from_uri_param(filt: &ExchangeFilter) -> ExchangeFilter {
        filt.clone()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ExchangeBalanceItem {
    pub currency: Currency,
    pub balance: i64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ExchangeAddress {
    pub currency: String,
    pub address: String,
    pub qr_code_base64: String,
}

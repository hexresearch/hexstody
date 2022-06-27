use base64;
use chrono::NaiveDateTime;
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

use super::domain::currency::{BtcAddress, Currency, CurrencyAddress};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceItem {
    pub currency: Currency,
    pub value: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositHistoryItem {
    pub currency: Currency,
    pub date: NaiveDateTime,
    pub value: i64,
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
    Confirmed,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositInfo {
    pub address: String,
}

/// Signature data that comes from operators
/// when they sign or reject requests.
/// This data type is used for authorization.
#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureData {
    pub signature: Signature,
    pub nonce: u64,
    pub public_key: PublicKey,
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
pub struct ConfirmationData(pub WithdrawalRequest);

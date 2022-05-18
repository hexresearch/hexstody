use chrono::NaiveDateTime;

use rocket::serde::uuid::Uuid;
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};

use super::domain::currency::{Currency, CurrencyAddress, BtcAddress};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum WithdrawalRequestStatus {
    UnderReview,
    InProgress,
    AwaitsApproval,
    Completed,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceItem {
    pub currency: Currency,
    pub value: i64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositHistoryItem {
    pub currency: Currency,
    pub date: NaiveDateTime,
    pub value: u64,
    pub number_of_confirmations: u64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WithdrawalHistoryItem {
    pub currency: Currency,
    pub date: NaiveDateTime,
    pub value: u64,
    pub status: WithdrawalRequestStatus,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum HistoryItem {
    Deposit(DepositHistoryItem),
    Withdrawal(WithdrawalHistoryItem),
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
    pub target_number_of_confirmations: u64,
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
    pub user: String,
    /// Receiving address
    pub address: CurrencyAddress,
    /// Amount of tokens to transfer
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
    #[schemars(example = "example_confrimtaion_status")]
    pub confrimtaion_status: String,
}

fn example_uuid() -> &'static str {
    "fdb12d51-0e3f-4ff8-821e-fbc255d8e413"
}

fn example_user() -> &'static str {
    "Alice"
}

fn example_address() -> CurrencyAddress {
    CurrencyAddress::BTC(BtcAddress("1BNwxHGaFbeUBitpjy2AsKpJ29Ybxntqvb".to_owned()))
}

fn example_datetime() -> &'static str {
    "2012-04-23T18:25:43.511Z"
}

fn example_amount() -> u64 {
    3
}

fn example_confrimtaion_status() -> &'static str {
    "1 of 3"
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DepositInfo {
    pub address: String,
}

use rocket::serde::uuid::Uuid;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use hexstody_db::domain::{BtcAddress, CurrencyAddress, EthAccount};
use hexstody_db::state::{WithdrawalRequest as WithdrawalRequestDb, WithdrawalRequestStatus};
use hexstody_db::update::withdrawal::WithdrawalRequestInfo as WithdrawalRequestInfoDb;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WithdrawalRequestInfo {
    /// User which initiated withdrawal request
    pub user: String,
    /// Receiving address
    pub address: String,
    /// Amount of tokens to transfer
    pub amount: u64,
}

impl Into<WithdrawalRequestInfoDb> for WithdrawalRequestInfo {
    fn into(self) -> WithdrawalRequestInfoDb {
        WithdrawalRequestInfoDb {
            user: self.user,
            address: CurrencyAddress::BTC(BtcAddress(self.address)),
            amount: self.amount,
        }
    }
}

/// Auxiliary data type to display `WithdrawalRequest` on the page
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalRequest {
    /// Request ID
    #[schemars(example = "example_uuid")]
    pub id: Uuid,
    /// User which initiated request
    #[schemars(example = "example_user")]
    pub user: String,
    /// Receiving address
    #[schemars(example = "example_address")]
    pub address: String,
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

fn example_address() -> &'static str {
    "1BNwxHGaFbeUBitpjy2AsKpJ29Ybxntqvb"
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

impl From<WithdrawalRequestDb> for WithdrawalRequest {
    fn from(withdrawal_request: WithdrawalRequestDb) -> Self {
        let address = match withdrawal_request.address.clone() {
            CurrencyAddress::BTC(BtcAddress(addr)) => addr,
            CurrencyAddress::ETH(EthAccount(addr)) => addr,
            CurrencyAddress::ERC20(_, EthAccount(addr)) => addr,
        };
        let confrimtaion_status = match withdrawal_request.confrimtaion_status.clone() {
            WithdrawalRequestStatus::NoConfirmationRequired => {
                "No confirmation required".to_owned()
            }
            WithdrawalRequestStatus::Confirmations(confirmations) => {
                format!("{} confirmations", confirmations.len())
            }
        };
        WithdrawalRequest {
            id: withdrawal_request.id,
            user: withdrawal_request.user.clone(),
            address: address,
            created_at: withdrawal_request
                .created_at
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            amount: withdrawal_request.amount,
            confrimtaion_status: confrimtaion_status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexHandlerContext {
    pub title: String,
    pub parent: String,
    pub withdrawal_requests: Vec<WithdrawalRequest>,
}

use crate::update::signup::UserId;
use crate::update::withdrawal::WithdrawalRequestInfo;
use chrono::prelude::*;
use ecdsa::{Signature, VerifyingKey};
use hexstody_api::domain::CurrencyAddress;
use p256::NistP256;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// It is unique withdrawal request ID whithin the system.
pub type WithdrawalRequestId = Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WithdrawalRequest {
    /// Request ID
    pub id: WithdrawalRequestId,
    /// User which initiated request
    pub user: UserId,
    /// Receiving address
    pub address: CurrencyAddress,
    /// When the request was created
    pub created_at: NaiveDateTime,
    /// Amount of tokens to transfer
    pub amount: u64,
    /// Some request require manual confirmation
    pub confrimtaion_status: WithdrawalRequestStatus,
}

impl From<(NaiveDateTime, WithdrawalRequestId, WithdrawalRequestInfo)> for WithdrawalRequest {
    fn from(value: (NaiveDateTime, WithdrawalRequestId, WithdrawalRequestInfo)) -> Self {
        WithdrawalRequest {
            id: value.1,
            user: value.2.user,
            address: value.2.address,
            created_at: value.0,
            amount: value.2.amount,
            confrimtaion_status: WithdrawalRequestStatus::Confirmations(Vec::new()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum WithdrawalRequestStatus {
    /// This request doesn't require manual confirmation
    NoConfirmationRequired,
    /// Vector of confirmations received from operators
    Confirmations(Vec<(VerifyingKey<NistP256>, Signature<NistP256>)>),
}

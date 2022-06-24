use crate::update::withdrawal::WithdrawalRequestInfo;
use crate::update::{signup::UserId, withdrawal::WithdrawalRequestDecision};
use hexstody_api::domain::CurrencyAddress;
use hexstody_api::types::{
    WithdrawalRequest as WithdrawalRequestApi,
    WithdrawalRequestStatus as WithdrawalRequestStatusApi,
};

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const REQUIRED_NUMBER_OF_CONFIRMATIONS: i16 = 2;

/// It is unique withdrawal request ID whithin the system.
pub type WithdrawalRequestId = Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum WithdrawalRequestStatus {
    /// Number of confirmations minus number of rejections received
    InProgress(i16),
    Confirmed,
    Rejected,
}

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
    pub status: WithdrawalRequestStatus,
    /// Confirmations received from operators
    pub confirmations: Vec<WithdrawalRequestDecision>,
    /// Rejections received from operators
    pub rejections: Vec<WithdrawalRequestDecision>,
}

impl From<(NaiveDateTime, WithdrawalRequestInfo)> for WithdrawalRequest {
    fn from(value: (NaiveDateTime, WithdrawalRequestInfo)) -> Self {
        WithdrawalRequest {
            id: value.1.id,
            user: value.1.user,
            address: value.1.address,
            created_at: value.0,
            amount: value.1.amount,
            status: WithdrawalRequestStatus::InProgress(0),
            confirmations: vec![],
            rejections: vec![],
        }
    }
}

impl Into<WithdrawalRequestApi> for WithdrawalRequest {
    fn into(self) -> WithdrawalRequestApi {
        let confirmation_status = match self.status {
            WithdrawalRequestStatus::InProgress(n) => {
                WithdrawalRequestStatusApi::InProgress { confirmations: n }
            }
            WithdrawalRequestStatus::Confirmed => WithdrawalRequestStatusApi::Confirmed,
            WithdrawalRequestStatus::Rejected => WithdrawalRequestStatusApi::Rejected,
        };
        WithdrawalRequestApi {
            id: self.id,
            user: self.user,
            address: self.address,
            created_at: self.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            amount: self.amount,
            confirmation_status: confirmation_status,
        }
    }
}

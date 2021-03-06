use crate::update::withdrawal::WithdrawalRequestInfo;
use crate::update::{signup::UserId, withdrawal::WithdrawalRequestDecision};
use hexstody_api::domain::{CurrencyAddress, CurrencyTxId};
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
    /// Confirmed by operators, but not yet sent to the node
    Confirmed,
    /// Tx sent to the node
    Completed {
        /// Time when the request was processed
        confirmed_at: NaiveDateTime,
        /// Txid
        txid: CurrencyTxId,
        /// Fee paid in sats. If an error occured, fee=0
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

impl Into<WithdrawalRequestStatusApi> for WithdrawalRequestStatus {
    fn into(self) -> WithdrawalRequestStatusApi {
        match self {
            WithdrawalRequestStatus::InProgress(n) => WithdrawalRequestStatusApi::InProgress { confirmations: n },
            WithdrawalRequestStatus::Confirmed => WithdrawalRequestStatusApi::Confirmed,
            WithdrawalRequestStatus::Completed { confirmed_at, txid, fee} => WithdrawalRequestStatusApi::Completed { confirmed_at, txid, fee},
            WithdrawalRequestStatus::OpRejected => WithdrawalRequestStatusApi::OpRejected,
            WithdrawalRequestStatus::NodeRejected{reason} => WithdrawalRequestStatusApi::NodeRejected{reason}
        }
    }
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
        let confirmation_status = self.status.into();
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

impl WithdrawalRequest {
    pub fn is_rejected(&self) -> bool {
        match self.status {
            WithdrawalRequestStatus::OpRejected => true,
            WithdrawalRequestStatus::NodeRejected{..} => true,
            _ => false
        }
    }

    /// Get fee for completed withdrawals, 0 for others
    pub fn fee(&self) -> u64 {
        match self.status {
            WithdrawalRequestStatus::Completed{fee, ..} => fee,
            _ => 0
        }
    }
}
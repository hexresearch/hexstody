use chrono::NaiveDateTime;
use p256::{ecdsa::Signature, PublicKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::withdraw::WithdrawalRequestId;
use crate::update::signup::UserId;
use hexstody_api::domain::{Currency, CurrencyAddress, CurrencyTxId};
use hexstody_api::types::{
    ConfirmationData, SignatureData, WithdrawalRequestInfo as WithdrawalRequestInfoApi, WithdrawalRequestDecisionType
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WithdrawalRequestInfo {
    /// Request ID
    pub id: WithdrawalRequestId,
    /// User which initiated withdrawal request
    pub user: UserId,
    /// Receiving address
    pub address: CurrencyAddress,
    /// Amount of tokens to transfer
    pub amount: u64,
}

impl From<WithdrawalRequestInfoApi> for WithdrawalRequestInfo {
    fn from(value: WithdrawalRequestInfoApi) -> WithdrawalRequestInfo {
        WithdrawalRequestInfo {
            id: Uuid::new_v4(),
            user: value.user,
            address: value.address,
            amount: value.amount,
        }
    }
}

// This data type is used to create DB state update
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WithdrawalRequestDecisionInfo {
    /// User which initiated withdrawal request
    pub user_id: UserId,
    /// Withdrawal request currency
    pub currency: Currency,
    /// Withdrawal request ID
    pub request_id: WithdrawalRequestId,
    /// API URL wich was used to send the decision
    pub url: String,
    /// Operator's digital signature
    pub signature: Signature,
    /// Nonce that was generated during decision
    pub nonce: u64,
    /// Operator's public key corresponding to the signing private key
    pub public_key: PublicKey,
    /// Decision type: confirm or reject
    pub decision_type: WithdrawalRequestDecisionType,
}

impl
    From<(
        ConfirmationData,
        SignatureData,
        WithdrawalRequestDecisionType,
        String,
    )> for WithdrawalRequestDecisionInfo
{
    fn from(
        value: (
            ConfirmationData,
            SignatureData,
            WithdrawalRequestDecisionType,
            String,
        ),
    ) -> WithdrawalRequestDecisionInfo {
        WithdrawalRequestDecisionInfo {
            user_id: value.0.user,
            currency: value.0.address.currency(),
            request_id: value.0.id,
            url: value.3,
            signature: value.1.signature,
            nonce: value.1.nonce,
            public_key: value.1.public_key,
            decision_type: value.2,
        }
    }
}

/// This data type is what actually stored in DB.
/// Contains information required to check operator's digital signature.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WithdrawalRequestDecision {
    /// API URL wich was used to send the decision
    pub url: String,
    /// Operator's digital signature
    pub signature: Signature,
    /// Nonce that was generated during decision
    pub nonce: u64,
    /// Operator's public key corresponding to the signing private key
    pub public_key: PublicKey,
}

impl From<WithdrawalRequestDecisionInfo> for WithdrawalRequestDecision {
    fn from(info: WithdrawalRequestDecisionInfo) -> WithdrawalRequestDecision {
        WithdrawalRequestDecision {
            url: info.url,
            signature: info.signature,
            nonce: info.nonce,
            public_key: info.public_key,
        }
    }
}

impl From<WithdrawalRequestDecision> for SignatureData {
    fn from(wrd: WithdrawalRequestDecision) -> SignatureData {
        SignatureData {
            signature: wrd.signature,
            nonce: wrd.nonce,
            public_key: wrd.public_key,
        }
    }
}

/// This data type is passed for an update.
/// Contains information required to set withdraw request to confirmed
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WithdrawCompleteInfo {
    pub id: WithdrawalRequestId,
    pub confirmed_at: NaiveDateTime,
    pub txid: CurrencyTxId
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WithdrawalRejectInfo {
    pub id: WithdrawalRequestId,
    pub reason: String
}
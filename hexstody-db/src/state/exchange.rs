use hexstody_api::{domain::Currency, types::{ExchangeStatus, SignatureData, ExchangeConfirmationData}};
use p256::{ecdsa::Signature, PublicKey};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

pub type ExchangeOrderId = Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ExchangeOrderUpd {
    pub id: ExchangeOrderId,
    pub user: String,
    pub currency_from: Currency,
    pub currency_to: Currency,
    pub amount: u64
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ExchangeOrder {
    pub id: ExchangeOrderId,
    pub user: String,
    pub currency_from: Currency,
    pub currency_to: Currency,
    pub amount: u64,
    pub status: ExchangeStatus,
    pub confirmations: Vec<SignatureData>,
    pub rejections: Vec<SignatureData>
}

impl ExchangeOrder {
    pub fn is_finalized(&self) -> bool {
        matches!(self.status, ExchangeStatus::Completed)
    }
    pub fn is_rejected(&self) -> bool {
        matches!(self.status, ExchangeStatus::Rejected)
    }
    pub fn is_pending(&self) -> bool {
        matches!(self.status, ExchangeStatus::InProgress {..})
    }
}

impl From<ExchangeOrder> for hexstody_api::types::ExchangeOrder{
    fn from(eo: ExchangeOrder) -> Self {
        let ExchangeOrder { id, user, currency_from, currency_to, amount, status, .. } = eo;
        hexstody_api::types::ExchangeOrder { user, id, currency_from, currency_to, amount, status }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ExchangeDecisionType {
    Confirm,
    Reject
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ExchangeDecision {
    /// User who initiated an exchange
    pub user: String,
    /// Exchange id
    pub id: Uuid,
    /// API URL wich was used to send the decision
    pub url: String,
    /// Operator's digital signature
    pub signature: Signature,
    /// Nonce that was generated during decision
    pub nonce: u64,
    /// Operator's public key corresponding to the signing private key
    pub public_key: PublicKey,
    /// Decision type: confirm or reject
    pub decision: ExchangeDecisionType,
}

impl
    From<(
        ExchangeConfirmationData,
        SignatureData,
        ExchangeDecisionType,
        String,
    )> for ExchangeDecision
{
    fn from(
        value: (
            ExchangeConfirmationData,
            SignatureData,
            ExchangeDecisionType,
            String,
        ),
    ) -> ExchangeDecision {
        ExchangeDecision {
            user: value.0.user,
            id: value.0.id,
            url: value.3,
            signature: value.1.signature,
            nonce: value.1.nonce,
            public_key: value.1.public_key,
            decision: value.2,
        }
    }
}

use hexstody_api::{domain::Currency, types::{Limit, LimitChangeStatus, SignatureData, LimitChangeResponse, LimitChangeDecisionType, LimitConfirmationData, LimitChangeFilter}};
use p256::{ecdsa::Signature, PublicKey};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct LimitCancelData{
    pub id: Uuid,
    pub user: String,
    pub currency: Currency
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct LimitChangeUpd{
    pub user: String,
    pub currency: Currency,
    pub limit: Limit
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LimitChangeData{
    pub id: Uuid,
    pub user: String,
    pub created_at: String,
    pub status: LimitChangeStatus,
    pub currency: Currency,
    pub limit: Limit,
    pub confirmations: Vec<SignatureData>,
    pub rejections: Vec<SignatureData>
}

impl Into<LimitChangeResponse> for LimitChangeData {
    fn into(self) -> LimitChangeResponse {
        let LimitChangeData{ id, user, created_at, status, currency, limit, .. } = self;
        LimitChangeResponse{ id, user, created_at, currency, limit ,status}
    }
}

impl LimitChangeData{
    pub fn has_already_signed(&self, pubkey: PublicKey) -> bool {
        let confirmed = self.confirmations.iter().any(|sd| sd.public_key == pubkey);
        if confirmed {return confirmed} else {
            return self.rejections.iter().any(|sd| sd.public_key == pubkey);
        }
    }

    pub fn has_confirmed(&self, pubkey: PublicKey) -> bool{
        self.confirmations.iter().any(|sd| sd.public_key == pubkey)
    }
    pub fn has_rejected(&self, pubkey: PublicKey) -> bool{
        self.rejections.iter().any(|sd| sd.public_key == pubkey)
    }

    pub fn matches_filter(&self, filter: LimitChangeFilter) -> bool{
        if matches!(filter, LimitChangeFilter::All) {
            true
        } else {
            match self.status{
                LimitChangeStatus::InProgress { .. } => matches!(filter, LimitChangeFilter::Pending),
                LimitChangeStatus::Completed => matches!(filter, LimitChangeFilter::Completed),
                LimitChangeStatus::Rejected => matches!(filter, LimitChangeFilter::Rejected),
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LimitChangeDecision{
    /// Limit change request id
    pub id: Uuid,
    /// User's id
    pub user: String,
    /// Currency to change limit for
    pub currency: Currency,
    /// Timestamp when the request was created
    pub created_at: String,
    /// Requested limit
    pub requested_limit: Limit,
    /// API URL wich was used to send the decision
    pub url: String,
    /// Operator's digital signature
    pub signature: Signature,
    /// Nonce that was generated during decision
    pub nonce: u64,
    /// Operator's public key corresponding to the signing private key
    pub public_key: PublicKey,
    /// Decision type: confirm or reject
    pub decision_type: LimitChangeDecisionType,
}

impl
    From<(
        LimitConfirmationData,
        SignatureData,
        LimitChangeDecisionType,
        String,
    )> for LimitChangeDecision
{
    fn from(
        value: (
            LimitConfirmationData,
            SignatureData,
            LimitChangeDecisionType,
            String,
        ),
    ) -> LimitChangeDecision {
        LimitChangeDecision {
            id: value.0.id,
            user: value.0.user,
            currency: value.0.currency,
            created_at: value.0.created_at,
            requested_limit: value.0.requested_limit,
            url: value.3,
            signature: value.1.signature,
            nonce: value.1.nonce,
            public_key: value.1.public_key,
            decision_type: value.2,
        }
    }
}
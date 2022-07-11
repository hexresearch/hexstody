use crate::state::withdraw::WithdrawalRequestId;
use crate::update::signup::UserId;
use serde::{Deserialize, Serialize};
use hexstody_api::domain::{CurrencyAddress};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WithdrawalRequestInfo0 {
    /// Request ID
    pub id: WithdrawalRequestId,
    /// User which initiated withdrawal request
    pub user: UserId,
    /// Receiving address
    pub address: CurrencyAddress,
    /// Amount of tokens to transfer
    pub amount: u64,
}
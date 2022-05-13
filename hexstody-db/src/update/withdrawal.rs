use serde::{Deserialize, Serialize};

use crate::update::signup::UserId;
use hexstody_api::domain::CurrencyAddress;
use hexstody_api::types::WithdrawalRequestInfo as WithdrawalRequestInfoApi;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WithdrawalRequestInfo {
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
            user: value.user,
            address: value.address,
            amount: value.amount,
        }
    }
}

use serde::{Deserialize, Serialize};

use crate::domain::CurrencyAddress;
use crate::update::signup::UserId;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WithdrawalRequestInfo {
    /// User which initiated withdrawal request
    pub user: UserId,
    /// Receiving address
    pub address: CurrencyAddress,
    /// Amount of tokens to transfer
    pub amount: u64,
}
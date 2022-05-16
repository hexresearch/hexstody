use super::signup::UserId;
use hexstody_api::domain::CurrencyAddress;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DepositAddress {
    /// It is unique user ID whithin the system. It is either email address or hex encoded LNAuth public key.
    pub user_id: UserId,
    /// Contains additional info that required to authentificated user in future.
    pub address: CurrencyAddress,
}

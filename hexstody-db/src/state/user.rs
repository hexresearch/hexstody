use super::transaction::*;
use super::withdraw::*;
use crate::update::signup::{SignupAuth, SignupInfo, UserId};
use chrono::prelude::*;
use hexstody_api::domain::{Currency, CurrencyAddress};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    /// It is unique user ID whithin the system. It is either email address or hex encoded LNAuth public key.
    pub username: UserId,
    /// Contains additional info that required to authentificated user in future.
    pub auth: SignupAuth,
    /// When the user was created
    pub created_at: NaiveDateTime,
    /// Withdrawal requests for given user. The id can be used to retreive the body of request.
    /// Here goes only withdrawals that are not yet fully performed.
    pub withdrawal_requests: HashSet<WithdrawalRequestId>,
    /// Information for each currency
    pub currencies: HashMap<Currency, UserCurrencyInfo>,
}

impl UserInfo {
    pub fn new(username: &str, auth: SignupAuth, created_at: NaiveDateTime) -> Self {
        UserInfo {
            username: username.to_owned(),
            auth,
            created_at,
            withdrawal_requests: HashSet::new(),
            currencies: Currency::supported()
                .into_iter()
                .map(|c| (c.clone(), UserCurrencyInfo::new(c)))
                .collect(),
        }
    }
}
impl From<(NaiveDateTime, SignupInfo)> for UserInfo {
    fn from(value: (NaiveDateTime, SignupInfo)) -> Self {
        UserInfo::new(&value.1.username, value.1.auth, value.0)
    }
}

/// User data for specific currency
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UserCurrencyInfo {
    /// Currency the info about
    pub currency: Currency,
    /// Required information for making deposit for the user in the specific currency.
    /// Oldest addresses goes last.
    pub deposit_info: Vec<CurrencyAddress>,
    /// Known set of transactions for the user, oldest transactions first.
    pub transactions: Vec<Transaction>,
    /// Users can create withdrawal requests that in some cases require manual confirmation from operators
    pub withdrawal_requests: HashMap<WithdrawalRequestId, WithdrawalRequest>,
}

impl UserCurrencyInfo {
    pub fn new(currency: Currency) -> Self {
        UserCurrencyInfo {
            currency,
            deposit_info: Vec::new(),
            transactions: Vec::new(),
            withdrawal_requests: HashMap::new(),
        }
    }
}

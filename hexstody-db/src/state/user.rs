use super::transaction::*;
use super::withdraw::*;
use crate::update::btc::BtcTxCancel;
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

    /// Return true if the user has given address as deposit address
    pub fn has_address(&self, address: &CurrencyAddress) -> bool {
        if let Some(cur_info) = self.currencies.get(&address.currency()) {
            cur_info.has_address(address)
        } else {
            false
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

    fn calculate_balance<F>(&self, btc_fee_per_transaction: u64, mut tx_filter: F) -> u64
    where
        F: FnMut(&Transaction) -> Option<&Transaction>,
    {
        let tx_sum: i64 = self
            .transactions
            .iter()
            .filter_map(|t| tx_filter(t).map(|t| t.amount()))
            .sum();
        let pending_withdrawals: u64 = self
            .withdrawal_requests
            .iter()
            .map(|(_, w)| w.amount - btc_fee_per_transaction)
            .sum();
        // zero to prevent spreading overflow bug when in less then out
        0.max(tx_sum - pending_withdrawals as i64) as u64
    }

    /// Includes unconfirmed transactions
    pub fn unconfirmed_transactions(&self) -> impl Iterator<Item = &Transaction> {
        self.transactions
            .iter()
            .filter_map(|t| if t.is_conflicted() { None } else { Some(t) })
    }

    /// Includes unconfirmed transactions
    pub fn balance(&self, btc_fee_per_transaction: u64) -> u64 {
        self.calculate_balance(btc_fee_per_transaction, |t| {
            if t.is_conflicted() {
                None
            } else {
                Some(t)
            }
        })
    }

    /// Include only finalized transactions
    pub fn finalized_balance(&self, btc_fee_per_transaction: u64) -> u64 {
        self.calculate_balance(btc_fee_per_transaction, |t| {
            if t.is_finalized() {
                None
            } else {
                Some(t)
            }
        })
    }

    pub fn has_address(&self, address: &CurrencyAddress) -> bool {
        self.deposit_info.iter().find(|a| *a == address).is_some()
    }

    pub fn update_btc_tx(&mut self, upd_tx: &BtcTransaction) {
        for tx in self.transactions.iter_mut() {
            match tx {
                Transaction::Btc(btc_tx) if btc_tx.is_same_btc_tx(upd_tx) => {
                    *btc_tx = upd_tx.clone();
                    return;
                }
                _ => (),
            }
        }
        self.transactions.push(Transaction::Btc(upd_tx.clone()));
    }

    pub fn cancel_btc_tx(&mut self, upd_tx: &BtcTxCancel) {
        let mut remove_i = None;
        for (i, tx) in self.transactions.iter().enumerate() {
            match tx {
                Transaction::Btc(btc_tx) if btc_tx.is_same_btc_tx(upd_tx) => {
                    remove_i = Some(i);
                    break;
                }
                _ => (),
            }
        }
        if let Some(i) = remove_i {
            self.transactions.remove(i);
        }
    }
}

use super::exchange::ExchangeOrder;
use super::exchange::ExchangeOrderId;
use super::transaction::*;
use super::withdraw::*;
use crate::update::btc::BtcTxCancel;
use crate::update::limit::LimitChangeData;
use crate::update::signup::{SignupAuth, SignupInfo, UserId};
use chrono::prelude::*;
use hexstody_api::domain::CurrencyTxId;
use hexstody_api::domain::Email;
use hexstody_api::domain::Language;
use hexstody_api::domain::PhoneNumber;
use hexstody_api::domain::TgName;
use hexstody_api::domain::{Currency, CurrencyAddress};
use hexstody_api::types::ExchangeFilter;
use hexstody_api::types::Invite;
use hexstody_api::types::LimitInfo;
use p256::PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UserConfig {
    pub language: Language,
    pub email: Option<Email>,
    pub phone: Option<PhoneNumber>,
    pub tg_name: Option<TgName>
}

impl Default for UserConfig {
    fn default() -> Self {
        Self { language: Language::English, email: Default::default(), phone: Default::default(), tg_name: Default::default() }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    /// It is unique user ID whithin the system. It is either email address or hex encoded LNAuth public key.
    pub username: UserId,
    /// User's invite
    pub invite: Invite,
    /// Contains additional info that required to authentificated user in future.
    pub auth: SignupAuth,
    /// When the user was created
    pub created_at: NaiveDateTime,
    /// Withdrawal requests for given user. The id can be used to retreive the body of request.
    /// Here goes only withdrawals that are not yet fully performed.
    pub withdrawal_requests: HashSet<WithdrawalRequestId>,
    /// Completed withdrawal requests
    pub completed_requests: HashSet<WithdrawalRequestId>,
    /// Information for each currency
    pub currencies: HashMap<Currency, UserCurrencyInfo>,
    /// Limit change requests
    pub limit_change_requests: HashMap<Currency, LimitChangeData>,
    /// User's config
    pub config: UserConfig,
    /// User's public key for public key authroization
    pub public_key: Option<PublicKey>
}

impl UserInfo {
    pub fn new(username: &str, invite: Invite, auth: SignupAuth, created_at: NaiveDateTime) -> Self {
        UserInfo {
            username: username.to_owned(),
            invite,
            auth,
            created_at,
            withdrawal_requests: HashSet::new(),
            completed_requests: HashSet::new(),
            currencies: Currency::default_currencies()
                .into_iter()
                .map(|c| (c.clone(), UserCurrencyInfo::new(c)))
                .collect(),
            limit_change_requests: HashMap::new(),
            config: UserConfig::default(),
            public_key: Option::default()
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

    pub fn find_completed_request(&self, txid: CurrencyTxId) -> Option<WithdrawalRequestId> {
        if let Some(cur_info) = self.currencies.get(&txid.currency()) {
            cur_info.find_completed_request(&txid)
        } else {None}
    }

    pub fn get_exchange_requests(&self, filter: ExchangeFilter) -> Vec<hexstody_api::types::ExchangeOrder> {
        self.currencies.values().flat_map(|c| 
            c.exchange_requests.values().filter_map(|eo| match filter {
                ExchangeFilter::All => Some(eo.clone().into()),
                ExchangeFilter::Completed => if eo.is_finalized() {Some(eo.clone().into())} else {None},
                ExchangeFilter::Rejected => if eo.is_rejected() {Some(eo.clone().into())} else {None},
                ExchangeFilter::Pending => if eo.is_pending() {Some(eo.clone().into())} else {None},
            })
        ).collect()
    }
}

impl From<(NaiveDateTime, SignupInfo)> for UserInfo {
    fn from(value: (NaiveDateTime, SignupInfo)) -> Self {
        UserInfo::new(&value.1.username, value.1.invite, value.1.auth, value.0)
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
    /// Users can create exchange requests that require manual confirmation from operators
    pub exchange_requests: HashMap<ExchangeOrderId, ExchangeOrder>,
    /// Confirmed incoming exchange requests. We store only amounts for balance calculations
    pub incoming_exchange_requests: HashMap<ExchangeOrderId, u64>,
    /// User's limit info. 
    pub limit_info: LimitInfo
}

impl UserCurrencyInfo {
    pub fn new(currency: Currency) -> Self {
        UserCurrencyInfo {
            currency,
            deposit_info: Vec::new(),
            transactions: Vec::new(),
            withdrawal_requests: HashMap::new(),
            exchange_requests: HashMap::new(),
            incoming_exchange_requests: HashMap::new(),
            limit_info: LimitInfo::default()
        }
    }

    /// Includes unconfirmed transactions
    pub fn unconfirmed_transactions(&self) -> impl Iterator<Item = &Transaction> {
        self.transactions
            .iter()
            .filter_map(|t| if t.is_conflicted() { None } else { Some(t) })
    }
    /// Includes unconfirmed transactions
    pub fn balance(&self) -> u64 {
        let tx_sum: i64 = self
            .transactions
            .iter()
            .filter_map(|t| {
                if t.is_conflicted() {
                    None
                } else {
                    Some(t.amount())
                }
            })
            .sum();
        // Do not count rejected withdrawals
        let pending_withdrawals: u64 = self.withdrawal_requests
            .iter()
            .map(|(_, w)|
                if w.is_rejected() {0}
                else {
                    w.amount + w.fee().unwrap_or(0)
                })
            .sum();
        let incoming: u64 = self.incoming_exchange_requests.values().sum();
        let outgoing: u64 = self.exchange_requests
            .values()
            .filter_map(|v| if v.is_rejected() {None} else {Some(v.amount_from)})
            .sum();
        let val = (incoming as i64) - (pending_withdrawals as i64) - (outgoing as i64);
        // zero to prevent spreading overflow bug when in less then out
        0.max(tx_sum + val) as u64
    }

    /// Include only finalized transactions
    pub fn finalized_balance(&self) -> u64 {
        let tx_sum: i64 = self
            .transactions
            .iter()
            .filter_map(|t| {
                if t.is_finalized() {
                    Some(t.amount())
                } else {
                    None
                }
            })
            .sum();
        // Do not count rejected withdrawals
        let pending_withdrawals: u64 = self.withdrawal_requests
            .iter()
            .map(|(_, w)|
                if w.is_rejected() {0}
                else {
                    w.amount + w.fee().unwrap_or(0)
                })
            .sum();
        let incoming: u64 = self.incoming_exchange_requests.values().sum();
        let outgoing: u64 = self.exchange_requests
            .values()
            .filter_map(|v| if v.is_rejected() {None} else {Some(v.amount_from)})
            .sum();
        let val = (incoming as i64) - (pending_withdrawals as i64) - (outgoing as i64);
        // zero to prevent spreading overflow bug when in less then out
        0.max(tx_sum + val) as u64
    }

    pub fn has_address(&self, address: &CurrencyAddress) -> bool {
        self.deposit_info.iter().find(|a| *a == address).is_some()
    }

    pub fn find_completed_request(&self, req_txid: &CurrencyTxId) -> Option<WithdrawalRequestId>{
        if req_txid.currency() == self.currency {
            self.withdrawal_requests
                .iter()
                .find_map(|(_, req)| {
                    match &req.status{
                        WithdrawalRequestStatus::Completed {txid, ..} => {
                            if req_txid.clone() == txid.clone() {
                                Some(req.id)
                            } else {None}
                        },
                        _ => None
                    }
                })
        }
        else {None}
    }

    pub fn update_btc_tx(&mut self, upd_tx: &BtcTransaction) {
        // Process only deposit transactions. Withdrawals are handled with WithdrawalRequests
        if upd_tx.amount >= 0 {
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

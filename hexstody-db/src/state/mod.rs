pub mod btc;
pub mod network;
pub mod transaction;
pub mod user;
pub mod withdraw;
pub mod exchange;

pub use btc::*;
use chrono::prelude::*;
use log::*;
pub use network::*;
use p256::PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
pub use transaction::*;
pub use user::*;
use uuid::Uuid;
pub use withdraw::*;

use crate::update::limit::{LimitCancelData, LimitChangeData, LimitChangeDecision, LimitChangeUpd};
use crate::update::misc::{
    ConfigUpdateData, InviteRec, PasswordChangeUpd, SetLanguage, SetPublicKey, TokenAction,
    TokenUpdate,
};
use crate::update::signup::SignupAuth;
use crate::update::withdrawal::{WithdrawCompleteInfo, WithdrawalRejectInfo};

use self::exchange::{ExchangeOrderUpd, ExchangeOrder, ExchangeDecision, ExchangeDecisionType, ExchangeState};

use super::update::btc::BtcTxCancel;
use super::update::deposit::DepositAddress;
use super::update::signup::{SignupInfo, UserId};
use super::update::withdrawal::{
    WithdrawalRequestDecision, WithdrawalRequestDecisionInfo, WithdrawalRequestInfo,
};
use super::update::{results::UpdateResult, StateUpdate, UpdateBody};
use hexstody_api::domain::*;
use hexstody_api::types::{
    WithdrawalRequestDecisionType, Invite, LimitChangeStatus, 
    LimitChangeDecisionType, SignatureData, LimitInfo, LimitSpan, 
    ExchangeStatus, ExchangeFilter, ExchangeOrder as ExchangeApiOrder};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct State {
    /// All known users of the system.
    pub users: HashMap<UserId, UserInfo>,
    /// Tracks when the state was last updated
    pub last_changed: NaiveDateTime,
    /// Tracks state of BTC chain
    pub btc_state: BtcState,
    /// Invites: Invite + string rep of pubk of the operator
    pub invites: HashMap<Invite, InviteRec>,
    /// Special wallet for exchanges
    pub exchange_state: ExchangeState
}

#[derive(Error, Debug, PartialEq)]
pub enum StateUpdateErr {
    #[error("User with ID {0} is already signed up")]
    UserAlreadyExists(UserId),
    #[error("User with ID {0} is not known")]
    UserNotFound(UserId),
    #[error("User {0} doesn't have currency {1}")]
    UserMissingCurrency(UserId, Currency),
    #[error("Deposit address {1} is already allocated for user {0}")]
    DepositAddressAlreadyAllocated(UserId, CurrencyAddress),
    #[error("User {0} doesn't have withdrawal request {1}")]
    WithdrawalRequestNotFound(UserId, WithdrawalRequestId),
    #[error("Withdrawal request {0} is already confirmed by {}", .1.to_string())]
    WithdrawalRequestConfirmationDuplicate(WithdrawalRequestId, PublicKey),
    #[error("Withdrawal request {0} is already rejected by {}", .1.to_string())]
    WithdrawalRequestRejectionDuplicate(WithdrawalRequestId, PublicKey),
    #[error("Withdrawal request {0} is already confirmed")]
    WithdrawalRequestAlreadyConfirmed(WithdrawalRequestId),
    #[error("Withdrawal request {0} is already rejected")]
    WithdrawalRequestAlreadyRejected(WithdrawalRequestId),
    #[error("{0} is already enabled")]
    TokenAlreadyEnabled(Erc20Token),
    #[error("{0} is already disabled")]
    TokenAlreadyDisabled(Erc20Token),
    #[error("{0} has non-zero balance. Can not disable")]
    TokenNonZeroBalance(Erc20Token),
    #[error("Failed to enable token {0} from {1}")]
    TokenEnableFail(Erc20Token, UserId),
    #[error("Invite already exist")]
    InviteAlreadyExist,
    #[error("Invite is not valid")]
    InviteNotFound,
    #[error("Limit request does not exist")]
    LimitChangeNotFound,
    #[error("Limit request already signed by the operator")]
    LimitAlreadySigned,
    #[error("Limit request already confirmed and finalized")]
    LimitAlreadyConfirmed,
    #[error("Limit request already rejected")]
    LimitAlreadyRejected,
    #[error("The spending is over the limit")]
    LimitOverflow,
    #[error("User {0} doesn't have enough of currency {1}")]
    InsufficientFunds(UserId, Currency),
    #[error("User {0} doesn't have outstanding exchange request for {1}")]
    UserMissingExchange(String, Currency),
    #[error("Exchange request already signed by the operator")]
    ExchangeAlreadySigned,
    #[error("Exchange request already confirmed and finalized")]
    ExchangeAlreadyConfirmed,
    #[error("Exchange request already rejected")]
    ExchangeAlreadyRejected,
}

impl State {
    pub fn new(network: Network) -> Self {
        State {
            users: HashMap::new(),
            last_changed: Utc::now().naive_utc(),
            btc_state: BtcState::new(network.btc()),
            invites: HashMap::new(),
            exchange_state: ExchangeState::new()
        }
    }

    /// Find user by attached deposit address
    pub fn find_user_address(&self, address: &CurrencyAddress) -> Option<UserId> {
        self.users
            .iter()
            .find(|(_, user)| user.has_address(address))
            .map(|(uid, _)| uid.clone())
    }

    /// Check if the address belongs to exchange wallet
    pub fn is_exchange_address(&self, address: &CurrencyAddress) -> bool {
        let currency = address.currency();
        let address = address.address();
        self.exchange_state.addresses.get(&currency).map(|addr| addr.address() == address).unwrap_or(false)
    }

    /// Find withdrawal by tx id
    pub fn get_user_by_id(&self, username: &str) -> Option<&UserInfo> {
        self.users.get(username)
    }

    pub fn find_withdrawal_by_tx_id(&self, txid: CurrencyTxId) -> Option<WithdrawalRequestId> {
        self.users
            .iter()
            .find_map(|(_, user)| user.find_completed_request(txid.clone()))
    }

    /// Apply an update event from persistent store
    pub fn apply_update(
        &mut self,
        update: StateUpdate,
    ) -> Result<Option<UpdateResult>, StateUpdateErr> {
        match update.body {
            UpdateBody::Signup(info) => {
                self.with_signup(update.created, info)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::Snapshot(snaphsot) => {
                *self = snaphsot;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::CreateWithdrawalRequest(withdrawal_request) => {
                let res = self.with_new_withdrawal_request(update.created, withdrawal_request)?;
                self.last_changed = update.created;
                info!("Res: {:?}", res);
                Ok(res)
            }
            UpdateBody::WithdrawalRequestDecision(withdrawal_request_decision) => {
                let res = self.with_withdrawal_request_decision(withdrawal_request_decision)?;
                self.last_changed = update.created;
                Ok(res.map(UpdateResult::WithdrawConfirmed))
            }
            UpdateBody::WithdrawalRequestComplete(withdrawal_completed_info) => {
                self.set_withdrawal_request_completed(withdrawal_completed_info)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::WithdrawalRequestNodeRejected(reject_info) => {
                self.set_withdrawal_request_node_rejected(reject_info)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::DepositAddress(dep_address) => {
                self.with_deposit_address(dep_address)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::BestBtcBlock(btc) => {
                self.btc_state = BtcState {
                    height: btc.height,
                    block_hash: btc.block_hash,
                };
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::UpdateBtcTx(tx) => {
                self.with_btc_tx_update(tx)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::CancelBtcTx(tx) => {
                self.with_btc_tx_cancel(tx)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::UpdateTokens(token_update) => {
                self.update_tokens(token_update)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::GenInvite(invite_req) => {
                self.gen_invite(invite_req)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::LimitsChangeRequest(req) => {
                self.insert_limits_req(req)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::CancelLimitChange(cancel_req) => {
                self.cancel_limit_change(cancel_req)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::LimitChangeDecision(limit_change_decision) => {
                self.with_limit_change_decision(limit_change_decision)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::ClearLimits(span) => {
                self.clear_limits(span)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::SetLanguage(req) => {
                self.set_language(req)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::ConfigUpdate(req) => {
                self.update_user_config(req)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::PasswordChange(req) => {
                self.change_password(req)?;
                self.last_changed = update.created;
                Ok(None)
            }
            UpdateBody::SetPublicKey(req) => {
                self.set_user_public_key(req)?;
                self.last_changed = update.created;
                Ok(None)
            },
            UpdateBody::ExchangeRequest(req) => {
                self.add_exchange_request(req)?;
                self.last_changed = update.created;
                Ok(None)
            },
            UpdateBody::ExchangeDecision(req) => {
                let b = self.apply_exchange_decision(&req)?;
                if b {
                    self.add_incoming_exchange(req)?;
                }
                self.last_changed = update.created;
                Ok(None)
            },
            UpdateBody::ExchangeAddress(req) => {
                self.set_exchange_address(req)?;
                self.last_changed = update.created;
                Ok(None)
            },
        }
    }

    /// Apply signup state update
    fn with_signup(
        &mut self,
        timestamp: NaiveDateTime,
        signup: SignupInfo,
    ) -> Result<(), StateUpdateErr> {
        let invite = signup.invite.clone();
        if self.users.contains_key(&signup.username) {
            return Err(StateUpdateErr::UserAlreadyExists(signup.username));
        }

        if !self.invites.contains_key(&invite) {
            return Err(StateUpdateErr::InviteNotFound);
        }
        let user_info: UserInfo = (timestamp, signup).into();
        self.users.insert(user_info.username.clone(), user_info);
        let _ = self.invites.remove(&invite);
        Ok(())
    }

    /// Apply new withdrawal request update
    fn with_new_withdrawal_request(
        &mut self,
        timestamp: NaiveDateTime,
        withdrawal_request_info: WithdrawalRequestInfo,
    ) -> Result<Option<UpdateResult>, StateUpdateErr> {
        let withdrawal_request: WithdrawalRequest =
            (timestamp, withdrawal_request_info.clone()).into();
        info!("withdrawal_request: {:?}", withdrawal_request);
        if let Some(user) = self.users.get_mut(&withdrawal_request.user) {
            let currency = withdrawal_request.address.currency();
            if let Some(cur_info) = user.currencies.get_mut(&currency) {
                match withdrawal_request.request_type {
                    WithdrawalRequestType::UnderLimit => {
                        if cur_info.limit_info.limit.amount
                            < cur_info.limit_info.spent + withdrawal_request.amount
                        {
                            return Err(StateUpdateErr::LimitOverflow);
                        } else {
                            cur_info
                                .withdrawal_requests
                                .insert(withdrawal_request_info.id, withdrawal_request.clone());
                            Ok(Some(UpdateResult::WithdrawalUnderlimit(
                                withdrawal_request.clone(),
                            )))
                        }
                    }
                    WithdrawalRequestType::OverLimit => {
                        cur_info
                            .withdrawal_requests
                            .insert(withdrawal_request_info.id, withdrawal_request);
                        Ok(None)
                    }
                }
            } else {
                Err(StateUpdateErr::UserMissingCurrency(
                    withdrawal_request.user,
                    currency,
                ))
            }
        } else {
            Err(StateUpdateErr::UserNotFound(withdrawal_request.user))
        }
    }

    fn get_withdrawal_request_by_decision_info(
        &mut self,
        withdrawal_request_decision: WithdrawalRequestDecisionInfo,
    ) -> Result<&mut WithdrawalRequest, StateUpdateErr> {
        if let Some(user) = self.users.get_mut(&withdrawal_request_decision.user_id) {
            if let Some(info) = user
                .currencies
                .get_mut(&withdrawal_request_decision.currency)
            {
                if let Some(withdrawal_request) = info
                    .withdrawal_requests
                    .get_mut(&withdrawal_request_decision.request_id)
                {
                    Ok(withdrawal_request)
                } else {
                    Err(StateUpdateErr::WithdrawalRequestNotFound(
                        withdrawal_request_decision.user_id,
                        withdrawal_request_decision.request_id,
                    ))
                }
            } else {
                Err(StateUpdateErr::UserMissingCurrency(
                    withdrawal_request_decision.user_id,
                    withdrawal_request_decision.currency,
                ))
            }
        } else {
            Err(StateUpdateErr::UserNotFound(
                withdrawal_request_decision.user_id,
            ))
        }
    }

    /// Apply withdrawal request decision update
    /// We don't check here that public key is in the whitelist,
    /// this is done by the web server.
    /// Returns Ok(True) if the request was just confirmed
    fn with_withdrawal_request_decision(
        &mut self,
        withdrawal_request_decision: WithdrawalRequestDecisionInfo,
    ) -> Result<Option<WithdrawalRequestId>, StateUpdateErr> {
        let withdrawal_request =
            self.get_withdrawal_request_by_decision_info(withdrawal_request_decision.clone())?;
        let is_confirmed_by_this_key = withdrawal_request
            .confirmations
            .iter()
            .any(|c| c.public_key == withdrawal_request_decision.public_key);
        let is_rejected_by_this_key = withdrawal_request
            .rejections
            .iter()
            .any(|c| c.public_key == withdrawal_request_decision.public_key);
        match withdrawal_request.status {
            WithdrawalRequestStatus::Completed { .. } => {
                return Err(StateUpdateErr::WithdrawalRequestAlreadyConfirmed(
                    withdrawal_request.id,
                ))
            }
            WithdrawalRequestStatus::Confirmed => {
                return Err(StateUpdateErr::WithdrawalRequestAlreadyConfirmed(
                    withdrawal_request.id,
                ))
            }
            WithdrawalRequestStatus::OpRejected => {
                return Err(StateUpdateErr::WithdrawalRequestAlreadyRejected(
                    withdrawal_request.id,
                ))
            }
            WithdrawalRequestStatus::NodeRejected { .. } => {
                return Err(StateUpdateErr::WithdrawalRequestAlreadyRejected(
                    withdrawal_request.id,
                ))
            }
            WithdrawalRequestStatus::InProgress(n) => {
                match withdrawal_request_decision.decision_type {
                    WithdrawalRequestDecisionType::Confirm => {
                        if is_confirmed_by_this_key {
                            return Err(StateUpdateErr::WithdrawalRequestConfirmationDuplicate(
                                withdrawal_request_decision.request_id,
                                withdrawal_request_decision.public_key,
                            ));
                        };
                        if is_rejected_by_this_key {
                            withdrawal_request
                                .rejections
                                .retain(|x| x.public_key != withdrawal_request_decision.public_key);
                        };
                        withdrawal_request
                            .confirmations
                            .push(WithdrawalRequestDecision::from(withdrawal_request_decision));
                        let m = if is_rejected_by_this_key { 2 } else { 1 };
                        if n == REQUIRED_NUMBER_OF_CONFIRMATIONS - m {
                            withdrawal_request.status = WithdrawalRequestStatus::Confirmed;
                            return Ok(Some(withdrawal_request.id));
                        } else {
                            withdrawal_request.status = WithdrawalRequestStatus::InProgress(n + m);
                            return Ok(None);
                        };
                    }
                    WithdrawalRequestDecisionType::Reject => {
                        if is_rejected_by_this_key {
                            return Err(StateUpdateErr::WithdrawalRequestRejectionDuplicate(
                                withdrawal_request_decision.request_id,
                                withdrawal_request_decision.public_key,
                            ));
                        };
                        if is_confirmed_by_this_key {
                            withdrawal_request
                                .confirmations
                                .retain(|x| x.public_key != withdrawal_request_decision.public_key);
                        };
                        withdrawal_request
                            .rejections
                            .push(WithdrawalRequestDecision::from(withdrawal_request_decision));
                        let m = if is_confirmed_by_this_key { 2 } else { 1 };
                        if n == m - REQUIRED_NUMBER_OF_CONFIRMATIONS {
                            withdrawal_request.status = WithdrawalRequestStatus::OpRejected;
                        } else {
                            withdrawal_request.status = WithdrawalRequestStatus::InProgress(n - m);
                        };
                        return Ok(None);
                    }
                }
            }
        };
    }

    /// Apply new withdrawal request update
    fn with_deposit_address(&mut self, dep_address: DepositAddress) -> Result<(), StateUpdateErr> {
        let user_id = &dep_address.user_id;
        if let Some(user) = self.users.get_mut(user_id) {
            let currency = dep_address.address.currency();
            if let Some(info) = user.currencies.get_mut(&currency) {
                if info.deposit_info.contains(&dep_address.address) {
                    Err(StateUpdateErr::DepositAddressAlreadyAllocated(
                        user_id.clone(),
                        dep_address.address,
                    ))
                } else {
                    info.deposit_info.push(dep_address.address);
                    Ok(())
                }
            } else {
                Err(StateUpdateErr::UserMissingCurrency(
                    user_id.clone(),
                    currency,
                ))
            }
        } else {
            Err(StateUpdateErr::UserNotFound(user_id.clone()))
        }
    }

    /// Apply update of BTC transaction
    fn with_btc_tx_update(&mut self, tx: BtcTransaction) -> Result<(), StateUpdateErr> {
        let address = CurrencyAddress::BTC(BtcAddress {
            addr: tx.address.to_string(),
        });
        if self.is_exchange_address(&address){
            self.exchange_state.process_incoming_btc_tx(tx);
            return Ok(())
        }
        if let Some(user_id) = self.find_user_address(&address) {
            if let Some(user) = self.users.get_mut(&user_id) {
                if let Some(curr_info) = user.currencies.get_mut(&Currency::BTC) {
                    curr_info.update_btc_tx(&tx);
                    Ok(())
                } else {
                    Err(StateUpdateErr::UserMissingCurrency(
                        user_id.clone(),
                        Currency::BTC,
                    ))
                }
            } else {
                Err(StateUpdateErr::UserNotFound(user_id.clone()))
            }
        } else {
            warn!("Unknown deposit address: {address}");
            Ok(())
        }
    }

    /// Apply cancel of BTC transaction
    fn with_btc_tx_cancel(&mut self, tx: BtcTxCancel) -> Result<(), StateUpdateErr> {
        let address = CurrencyAddress::BTC(BtcAddress {
            addr: tx.address.0.to_string(),
        });
        if self.is_exchange_address(&address) {
            self.exchange_state.cancel_btc_tx(tx);
            return Ok(());
        }
        let res1 = if let Some(user_id) = self.find_user_address(&address) {
            if let Some(user) = self.users.get_mut(&user_id) {
                if let Some(curr_info) = user.currencies.get_mut(&Currency::BTC) {
                    curr_info.cancel_btc_tx(&tx);
                    Ok(())
                } else {
                    Err(StateUpdateErr::UserMissingCurrency(
                        user_id.clone(),
                        Currency::BTC,
                    ))
                }
            } else {
                Err(StateUpdateErr::UserNotFound(user_id.clone()))
            }
        } else {
            warn!("Unknown deposit address: {address}");
            Ok(())
        };

        let txid = CurrencyTxId::BTC(BTCTxid {
            txid: tx.txid.0.to_string(),
        });
        let res2 = if let Some(rid) = self.find_withdrawal_by_tx_id(txid) {
            let reject = WithdrawalRejectInfo {
                id: rid,
                reason: "Tx canceled".to_owned(),
            };
            self.set_withdrawal_request_node_rejected(reject)
        } else {
            Ok(())
        };
        if res1.is_err() && res2.is_err() {
            res1
        } else {
            Ok(())
        }
    }

    /// Take ordered chain of updates and collect the accumulated state.
    /// Order should be from the earliest to the latest.
    pub fn collect<I>(network: Network, updates: I) -> Result<Self, StateUpdateErr>
    where
        I: IntoIterator<Item = StateUpdate>,
    {
        let mut state = State::new(network);
        for upd in updates.into_iter() {
            state.apply_update(upd)?;
        }
        Ok(state)
    }

    /// Extract all pending withdrawal requests
    pub fn withdrawal_requests(&self) -> HashMap<WithdrawalRequestId, WithdrawalRequest> {
        let mut result = HashMap::new();
        for (_, user) in self.users.iter() {
            for (_, info) in user.currencies.iter() {
                for (req_id, req) in info.withdrawal_requests.iter() {
                    result.insert(req_id.clone(), req.clone());
                }
            }
        }
        result
    }

    pub fn get_withdrawal_request(&self, id: WithdrawalRequestId) -> Option<WithdrawalRequest> {
        for (_, user) in self.users.iter() {
            for (_, info) in user.currencies.iter() {
                for (req_id, req) in info.withdrawal_requests.iter() {
                    if req_id.clone() == id {
                        return Some(req.clone());
                    }
                }
            }
        }
        None
    }

    pub fn set_withdrawal_request_completed(
        &mut self,
        withdrawal_confirmed_info: WithdrawCompleteInfo,
    ) -> Result<(), StateUpdateErr> {
        for (_, user) in self.users.iter_mut() {
            for (_, info) in user.currencies.iter_mut() {
                for (req_id, req) in info.withdrawal_requests.iter_mut() {
                    if req_id.clone() == withdrawal_confirmed_info.id {
                        let stat = WithdrawalRequestStatus::Completed {
                            confirmed_at: withdrawal_confirmed_info.confirmed_at,
                            txid: withdrawal_confirmed_info.txid.clone(),
                            fee: withdrawal_confirmed_info.fee,
                            input_addresses: withdrawal_confirmed_info.input_addresses.clone(),
                            output_addresses: withdrawal_confirmed_info.output_addresses.clone(),
                        };
                        req.status = stat;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn set_withdrawal_request_node_rejected(
        &mut self,
        reject_info: WithdrawalRejectInfo,
    ) -> Result<(), StateUpdateErr> {
        for (_, user) in self.users.iter_mut() {
            for (_, info) in user.currencies.iter_mut() {
                for (req_id, req) in info.withdrawal_requests.iter_mut() {
                    if req_id.clone() == reject_info.id {
                        let stat = WithdrawalRequestStatus::NodeRejected {
                            reason: reject_info.reason.clone(),
                        };
                        req.status = stat;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn update_tokens(&mut self, token_update: TokenUpdate) -> Result<(), StateUpdateErr> {
        let TokenUpdate {
            user,
            token,
            action,
        } = token_update;
        match self.users.get_mut(&user) {
            None => Err(StateUpdateErr::UserNotFound(user)),
            Some(user_info) => {
                let cur = Currency::ERC20(token.clone());
                match action {
                    TokenAction::Enable => {
                        if user_info.currencies.contains_key(&cur) {
                            Err(StateUpdateErr::TokenAlreadyEnabled(token))
                        } else {
                            user_info
                                .currencies
                                .insert(cur.clone(), UserCurrencyInfo::new(cur.clone()));
                            Ok(())
                        }
                    }
                    TokenAction::Disable => match user_info.currencies.get(&cur) {
                        Some(info) => {
                            if info.balance() > 0 {
                                Err(StateUpdateErr::TokenNonZeroBalance(token))
                            } else {
                                user_info.currencies.remove(&cur);
                                user_info.limit_change_requests.remove(&cur);
                                Ok(())
                            }
                        }
                        None => Err(StateUpdateErr::TokenAlreadyDisabled(token)),
                    },
                }
            }
        }
    }

    fn gen_invite(&mut self, invite_req: InviteRec) -> Result<(), StateUpdateErr> {
        let invite = invite_req.invite;
        if let Some(_) = self.invites.get(&invite) {
            Err(StateUpdateErr::InviteAlreadyExist)
        } else {
            self.invites.insert(invite, invite_req);
            Ok(())
        }
    }

    fn insert_limits_req(&mut self, req: LimitChangeUpd) -> Result<(), StateUpdateErr> {
        let cur = req.currency.clone();
        match self.users.get_mut(&req.user) {
            Some(usr) => match usr.currencies.get_mut(&cur) {
                None => Err(StateUpdateErr::UserMissingCurrency(req.user, cur)),
                Some(_) => {
                    let data = LimitChangeData {
                        id: Uuid::new_v4(),
                        user: usr.username.clone(),
                        created_at: chrono::offset::Utc::now().to_string(),
                        status: LimitChangeStatus::InProgress {
                            confirmations: 0,
                            rejections: 0,
                        },
                        currency: cur.clone(),
                        limit: req.limit,
                        confirmations: vec![],
                        rejections: vec![],
                    };
                    usr.limit_change_requests.insert(cur, data);
                    Ok(())
                }
            },
            None => Err(StateUpdateErr::UserNotFound(req.user)),
        }
    }

    fn cancel_limit_change(&mut self, cancel_req: LimitCancelData) -> Result<(), StateUpdateErr> {
        let LimitCancelData {
            id: _,
            user,
            currency,
        } = cancel_req;
        match self.users.get_mut(&user) {
            Some(usr) => {
                let _ = usr.limit_change_requests.remove(&currency);
                Ok(())
            }
            None => Err(StateUpdateErr::UserNotFound(user)),
        }
    }

    fn with_limit_change_decision(
        &mut self,
        lcd: LimitChangeDecision,
    ) -> Result<(), StateUpdateErr> {
        match self.users.get_mut(&lcd.user) {
            Some(usr) => match usr.limit_change_requests.get_mut(&lcd.currency) {
                Some(req) => {
                    let sdata = SignatureData {
                        signature: lcd.signature,
                        nonce: lcd.nonce,
                        public_key: lcd.public_key,
                    };
                    match req.status {
                        LimitChangeStatus::Completed => Err(StateUpdateErr::LimitAlreadyConfirmed),
                        LimitChangeStatus::Rejected => Err(StateUpdateErr::LimitAlreadyRejected),
                        LimitChangeStatus::InProgress {
                            confirmations,
                            rejections,
                        } => match lcd.decision_type {
                            LimitChangeDecisionType::Confirm => {
                                if req.has_confirmed(lcd.public_key) {
                                    Err(StateUpdateErr::LimitAlreadyConfirmed)
                                } else {
                                    req.confirmations.push(sdata);
                                    if confirmations + 1 - rejections >= 2 {
                                        req.status = LimitChangeStatus::Completed;
                                        if let Some(cinfo) = usr.currencies.get_mut(&lcd.currency) {
                                            cinfo.limit_info = LimitInfo {
                                                limit: lcd.requested_limit.clone(),
                                                spent: 0,
                                            }
                                        }
                                        let _ = usr.limit_change_requests.remove(&lcd.currency);
                                    } else {
                                        req.status = LimitChangeStatus::InProgress {
                                            confirmations: confirmations + 1,
                                            rejections,
                                        };
                                    };
                                    Ok(())
                                }
                            }
                            LimitChangeDecisionType::Reject => {
                                if req.has_rejected(lcd.public_key) {
                                    return Err(StateUpdateErr::LimitAlreadyConfirmed);
                                } else {
                                    req.rejections.push(sdata);
                                    if rejections + 1 - confirmations >= 2 {
                                        req.status = LimitChangeStatus::Rejected;
                                        let _ = usr.limit_change_requests.remove(&lcd.currency);
                                    } else {
                                        req.status = LimitChangeStatus::InProgress {
                                            confirmations,
                                            rejections: rejections + 1,
                                        };
                                    };
                                    Ok(())
                                }
                            }
                        },
                    }
                }
                None => Err(StateUpdateErr::LimitChangeNotFound),
            },
            None => Err(StateUpdateErr::UserNotFound(lcd.user)),
        }
    }

    fn clear_limits(&mut self, span: LimitSpan) -> Result<(), StateUpdateErr> {
        for (_, uinfo) in self.users.iter_mut() {
            for curinfo in uinfo.currencies.values_mut() {
                if curinfo.limit_info.limit.span == span {
                    curinfo.limit_info.spent = 0;
                }
            }
        }
        Ok(())
    }

    fn set_language(&mut self, req: SetLanguage) -> Result<(), StateUpdateErr> {
        match self.users.get_mut(&req.user) {
            Some(uinfo) => {
                uinfo.config.language = req.language;
                Ok(())
            }
            None => Err(StateUpdateErr::UserNotFound(req.user)),
        }
    }

    fn update_user_config(&mut self, req: ConfigUpdateData) -> Result<(), StateUpdateErr> {
        let ConfigUpdateData {
            user,
            email,
            phone,
            tg_name,
        } = req;
        match self.users.get_mut(&user) {
            Some(uinfo) => {
                if let Some(email) = email {
                    uinfo.config.email = email.ok();
                }
                if let Some(phone) = phone {
                    uinfo.config.phone = phone.ok();
                }
                if let Some(tg_name) = tg_name {
                    uinfo.config.tg_name = tg_name.ok();
                }
                Ok(())
            }
            None => Err(StateUpdateErr::UserNotFound(user)),
        }
    }

    fn change_password(&mut self, req: PasswordChangeUpd) -> Result<(), StateUpdateErr> {
        let PasswordChangeUpd { user, new_password } = req;
        match self.users.get_mut(&user) {
            Some(uinfo) => {
                uinfo.auth = SignupAuth::Password(new_password);
                Ok(())
            }
            None => Err(StateUpdateErr::UserNotFound(user)),
        }
    }

    fn set_user_public_key(&mut self, req: SetPublicKey) -> Result<(), StateUpdateErr> {
        let SetPublicKey { user, public_key } = req;
        match self.users.get_mut(&user) {
            Some(uinfo) => {
                uinfo.public_key = public_key;
                Ok(())
            }
            None => Err(StateUpdateErr::UserNotFound(user)),
        }
    }

    fn add_exchange_request(&mut self, req: ExchangeOrderUpd) -> Result<(), StateUpdateErr> {
        let ExchangeOrderUpd { user, currency_from, currency_to, amount_from, amount_to, id, created_at } = req;
        let uinfo = self.users.get_mut(&user).ok_or(StateUpdateErr::UserNotFound(user.clone()))?;
        let cinfo = uinfo.currencies.get_mut(&currency_from).ok_or(StateUpdateErr::UserMissingCurrency(user.clone(), currency_from.clone()))?;
        if cinfo.balance() < amount_from {
            return Err(StateUpdateErr::InsufficientFunds(user.clone(), currency_from.clone()))
        }
        let order = ExchangeOrder {
            id: id.clone(),
            user,
            currency_from,
            currency_to,
            amount_from,
            amount_to,
            status: ExchangeStatus::InProgress { confirmations: 0, rejections: 0 },
            confirmations: Vec::new(),
            rejections: Vec::new(),
            created_at, 
        };
        cinfo.exchange_requests.insert(id, order);
        Ok(())
    }

    pub const EXCHANGE_NUMBER_OF_CONFIRMATIONS: i16 = 1;

    /// Returns true if we need to update the balance for the target currency
    /// This is required since we can't borrow user info as mutable twice
    fn apply_exchange_decision(&mut self, req: &ExchangeDecision) -> Result<bool, StateUpdateErr> {
        let user = req.user.clone();
        let currency_from = req.currency_from.clone();
        let uinfo = self.users.get_mut(&user).ok_or(StateUpdateErr::UserNotFound(user.clone()))?;
        let cinfo = uinfo.currencies.get_mut(&currency_from).ok_or(StateUpdateErr::UserMissingCurrency(user.clone(), currency_from.clone()))?;
        let exchange = cinfo.exchange_requests.get_mut(&req.id).ok_or(StateUpdateErr::UserMissingExchange(user.clone(), currency_from.clone()))?;
        let sdata = SignatureData{ signature: req.signature, nonce: req.nonce, public_key: req.public_key };
        match exchange.status {
            ExchangeStatus::Completed => Err(StateUpdateErr::ExchangeAlreadyConfirmed),
            ExchangeStatus::Rejected => Err(StateUpdateErr::ExchangeAlreadyRejected),
            ExchangeStatus::InProgress { confirmations, rejections } => match req.decision {
                ExchangeDecisionType::Confirm => if exchange.has_confirmed(req.public_key) {
                    Err(StateUpdateErr::ExchangeAlreadyConfirmed)
                } else {
                    exchange.confirmations.push(sdata);
                    if confirmations + 1 - rejections >= State::EXCHANGE_NUMBER_OF_CONFIRMATIONS {
                        exchange.status = ExchangeStatus::Completed;
                        self.exchange_state.process_order(exchange.into_exchange_upd());
                        Ok(true)
                    } else {
                        exchange.status = ExchangeStatus::InProgress { confirmations: confirmations + 1, rejections: rejections };
                        Ok(false)
                    }
                },
                ExchangeDecisionType::Reject => if exchange.has_rejected(req.public_key) {
                    Err(StateUpdateErr::ExchangeAlreadyRejected)
                } else {
                    exchange.confirmations.push(sdata);
                    if rejections + 1 - confirmations >= State::EXCHANGE_NUMBER_OF_CONFIRMATIONS {
                        exchange.status = ExchangeStatus::Rejected
                    } else {
                        exchange.status = ExchangeStatus::InProgress { confirmations: confirmations, rejections: rejections + 1}
                    }
                    Ok(false)
                },
            }
        }
    }

    fn add_incoming_exchange(&mut self, req: ExchangeDecision) -> Result<(), StateUpdateErr> {
        let uinfo = self.users.get_mut(&req.user).ok_or(StateUpdateErr::UserNotFound(req.user.clone()))?;
        let cinfo = uinfo.currencies.get_mut(&req.currency_to).ok_or(StateUpdateErr::UserMissingCurrency(req.user.clone(), req.currency_to.clone()))?;
        cinfo.incoming_exchange_requests.insert(req.id, req.amount_to);
        Ok(())
    }

    pub fn get_exchange_requests(&self, filter: ExchangeFilter) -> Vec<ExchangeApiOrder>{
        self.users.values().flat_map(|u| u.get_exchange_requests(filter)).collect()
    }

    fn set_exchange_address(&mut self, req: CurrencyAddress) -> Result<(), StateUpdateErr> {
        self.exchange_state.addresses.insert(req.currency(), req.clone());
        Ok(())
    } 
}

impl Default for State {
    fn default() -> Self {
        State::new(Network::Mainnet)
    }
}

#[cfg(test)]
mod tests {
    use p256::{
        ecdsa::{signature::Signer, SigningKey},
        SecretKey,
    };
    use rand_core::OsRng;
    use sqlx::{Pool, Postgres};
    use uuid::Uuid;

    use super::*;
    use crate::queries::*;
    use crate::update::signup::{SignupAuth, SignupInfo};
    use crate::update::StateUpdate;
    use hexstody_api::domain::{BtcAddress, CurrencyAddress};

    async fn apply_state_update(
        update: StateUpdate,
        state: &mut State,
        pool: &Pool<Postgres>,
    ) -> NaiveDateTime {
        insert_update(pool, update.body.clone(), Some(update.created))
            .await
            .unwrap();
        state.apply_update(update.clone()).unwrap();
        return update.created;
    }

    #[sqlx_database_tester::test(pool(variable = "pool", migrations = "./migrations"))]
    async fn test_signup_update() {
        let mut state = State::default();
        let invite = Invite {
            invite: Uuid::new_v4(),
        };
        let invite_rec = InviteRec {
            invite: invite.clone(),
            invitor: String::new(),
            label: String::new(),
        };
        let _ = apply_state_update(
            StateUpdate::new(UpdateBody::GenInvite(invite_rec.clone())),
            &mut state,
            &pool,
        )
        .await;
        let signup_info = SignupInfo {
            username: "Alice".to_owned(),
            invite,
            auth: SignupAuth::Lightning,
        };
        let created_at = apply_state_update(
            StateUpdate::new(UpdateBody::Signup(signup_info.clone())),
            &mut state,
            &pool,
        )
        .await;
        let state = query_state(Network::Regtest, &pool).await.unwrap();
        let expected_user = UserInfo::from((created_at, signup_info.clone()));
        let extracted_user = state
            .users
            .get(&signup_info.username)
            .cloned()
            .map(|mut u| {
                u.created_at = created_at;
                u
            });
        assert_eq!(extracted_user, Some(expected_user));
    }

    #[sqlx_database_tester::test(pool(variable = "pool", migrations = "./migrations"))]
    async fn test_new_withdrawal_request_update() {
        let mut state = State::default();
        let invite = Invite {
            invite: Uuid::new_v4(),
        };
        let invite_rec = InviteRec {
            invite: invite.clone(),
            invitor: String::new(),
            label: String::new(),
        };
        let _ = apply_state_update(
            StateUpdate::new(UpdateBody::GenInvite(invite_rec.clone())),
            &mut state,
            &pool,
        )
        .await;
        let signup_info = SignupInfo {
            username: "Alice".to_owned(),
            invite,
            auth: SignupAuth::Lightning,
        };
        let withdrawal_request_info = WithdrawalRequestInfo {
            id: Uuid::new_v4(),
            user: signup_info.username.clone(),
            address: CurrencyAddress::BTC(BtcAddress {
                addr: "bc1qpv8tczdsft9lmlz4nhz8058jdyl96velqqlwgj".to_owned(),
            }),
            amount: 1,
            request_type: WithdrawalRequestType::OverLimit,
        };
        let _ = apply_state_update(
            StateUpdate::new(UpdateBody::Signup(signup_info.clone())),
            &mut state,
            &pool,
        )
        .await;
        let _ = apply_state_update(
            StateUpdate::new(UpdateBody::CreateWithdrawalRequest(
                withdrawal_request_info.clone(),
            )),
            &mut state,
            &pool,
        )
        .await;
        let state = query_state(Network::Regtest, &pool).await.unwrap();
        let extracted_withdrawal_request = state
            .users
            .get(&signup_info.username.clone())
            .unwrap()
            .currencies
            .get(&withdrawal_request_info.address.clone().currency())
            .unwrap()
            .withdrawal_requests
            .get(&withdrawal_request_info.id.clone())
            .unwrap();
        assert_eq!(
            *extracted_withdrawal_request,
            WithdrawalRequest {
                id: extracted_withdrawal_request.id,
                user: withdrawal_request_info.user,
                address: withdrawal_request_info.address,
                created_at: extracted_withdrawal_request.created_at,
                amount: withdrawal_request_info.amount,
                status: WithdrawalRequestStatus::InProgress(0),
                confirmations: vec![],
                rejections: vec![],
                request_type: WithdrawalRequestType::OverLimit
            }
        );
    }

    #[sqlx_database_tester::test(pool(variable = "pool", migrations = "./migrations"))]
    async fn test_new_withdrawal_request_decision_update() {
        let mut state = State::default();
        let invite = Invite {
            invite: Uuid::new_v4(),
        };
        let invite_rec = InviteRec {
            invite: invite.clone(),
            invitor: String::new(),
            label: String::new(),
        };
        let _ = apply_state_update(
            StateUpdate::new(UpdateBody::GenInvite(invite_rec.clone())),
            &mut state,
            &pool,
        )
        .await;
        let signup_info = SignupInfo {
            username: "Alice".to_owned(),
            invite,
            auth: SignupAuth::Lightning,
        };
        let withdrawal_request_info = WithdrawalRequestInfo {
            id: Uuid::new_v4(),
            user: signup_info.username.clone(),
            address: CurrencyAddress::BTC(BtcAddress {
                addr: "bc1qpv8tczdsft9lmlz4nhz8058jdyl96velqqlwgj".to_owned(),
            }),
            amount: 1,
            request_type: WithdrawalRequestType::OverLimit,
        };
        let _ = apply_state_update(
            StateUpdate::new(UpdateBody::Signup(signup_info.clone())),
            &mut state,
            &pool,
        )
        .await;
        let _ = apply_state_update(
            StateUpdate::new(UpdateBody::CreateWithdrawalRequest(
                withdrawal_request_info.clone(),
            )),
            &mut state,
            &pool,
        )
        .await;
        let extracted_withdrawal_request = state
            .users
            .get(&signup_info.username.clone())
            .unwrap()
            .currencies
            .get(&withdrawal_request_info.address.clone().currency())
            .unwrap()
            .withdrawal_requests
            .get(&withdrawal_request_info.id.clone())
            .unwrap();
        let url = "test".to_owned();
        let extracted_withdrawal_request_json =
            serde_json::to_string(extracted_withdrawal_request).unwrap();
        let nonce = 0;
        let msg = [url, extracted_withdrawal_request_json, nonce.to_string()].join(":");
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        let signature = SigningKey::from(secret_key).sign(msg.as_bytes());
        let withdrawal_request_decision_info = WithdrawalRequestDecisionInfo {
            user_id: signup_info.username.clone(),
            currency: withdrawal_request_info.address.clone().currency(),
            request_id: extracted_withdrawal_request.id,
            url: "test".to_owned(),
            nonce: nonce,
            signature: signature,
            public_key: public_key,
            decision_type: WithdrawalRequestDecisionType::Confirm,
        };
        let _ = apply_state_update(
            StateUpdate::new(UpdateBody::WithdrawalRequestDecision(
                withdrawal_request_decision_info.clone(),
            )),
            &mut state,
            &pool,
        )
        .await;
        let state = query_state(Network::Regtest, &pool).await.unwrap();
        let extracted_withdrawal_request = state
            .users
            .get(&signup_info.username.clone())
            .unwrap()
            .currencies
            .get(&withdrawal_request_info.address.clone().currency())
            .unwrap()
            .withdrawal_requests
            .get(&withdrawal_request_info.id.clone())
            .unwrap();
        assert_eq!(
            extracted_withdrawal_request.status,
            WithdrawalRequestStatus::InProgress(1)
        );
        assert_eq!(
            extracted_withdrawal_request.confirmations,
            vec![WithdrawalRequestDecision::from(
                withdrawal_request_decision_info
            )]
        );
    }
}

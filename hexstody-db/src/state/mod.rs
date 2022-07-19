pub mod btc;
pub mod network;
pub mod transaction;
pub mod user;
pub mod withdraw;

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
pub use withdraw::*;

use crate::update::withdrawal::{WithdrawCompleteInfo, WithdrawalRejectInfo};

use super::update::btc::BtcTxCancel;
use super::update::deposit::DepositAddress;
use super::update::signup::{SignupInfo, UserId};
use super::update::withdrawal::{
    WithdrawalRequestDecision, WithdrawalRequestDecisionInfo,
    WithdrawalRequestInfo,
};
use super::update::{StateUpdate, UpdateBody, results::UpdateResult};
use hexstody_api::domain::*;
use hexstody_api::types::WithdrawalRequestDecisionType;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct State {
    /// All known users of the system.
    /// TODO: There is possible DDoS attack on signup of million of users.
    ///     We need to implement rate limits for it and auto cleanup of unused empty accounts.
    pub users: HashMap<UserId, UserInfo>,
    /// Tracks when the state was last updated
    pub last_changed: NaiveDateTime,
    /// Tracks state of BTC chain
    pub btc_state: BtcState,
}

#[derive(Error, Debug, PartialEq)]
pub enum StateUpdateErr {
    #[error("User with ID {0} is already signed up")]
    UserAlreadyExists(UserId),
    #[error("User with ID {0} is not known")]
    UserNotFound(UserId),
    #[error("User {0} doesn't have currency {1}")]
    UserMissingCurrency(UserId, Currency),
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
}

impl State {
    pub fn new(network: Network) -> Self {
        State {
            users: HashMap::new(),
            last_changed: Utc::now().naive_utc(),
            btc_state: BtcState::new(network.btc()),
        }
    }

    /// Find user by attached deposit address
    pub fn find_user_address(&self, address: &CurrencyAddress) -> Option<UserId> {
        self.users
            .iter()
            .find(|(_, user)| user.has_address(address))
            .map(|(uid, _)| uid.clone())
    }

    pub fn find_withdrawal_by_tx_id(&self, txid: CurrencyTxId) -> Option<WithdrawalRequestId>{
        self.users
            .iter()
            .find_map(|(_, user)| 
                user.find_completed_request(txid.clone())
            )
    }

    /// Apply an update event from persistent store
    pub fn apply_update(&mut self, update: StateUpdate) -> Result<Option<UpdateResult>, StateUpdateErr> {
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
                self.with_new_withdrawal_request(update.created, withdrawal_request)?;
                self.last_changed = update.created;
                Ok(None)
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
        }
    }

    /// Apply signup state update
    fn with_signup(
        &mut self,
        timestamp: NaiveDateTime,
        signup: SignupInfo,
    ) -> Result<(), StateUpdateErr> {
        if self.users.contains_key(&signup.username) {
            return Err(StateUpdateErr::UserAlreadyExists(signup.username));
        }

        let user_info: UserInfo = (timestamp, signup).into();
        self.users.insert(user_info.username.clone(), user_info);

        Ok(())
    }

    /// Apply new withdrawal request update
    fn with_new_withdrawal_request(
        &mut self,
        timestamp: NaiveDateTime,
        withdrawal_request_info: WithdrawalRequestInfo,
    ) -> Result<(), StateUpdateErr> {
        let withdrawal_request: WithdrawalRequest =
            (timestamp, withdrawal_request_info.clone()).into();
        if let Some(user) = self.users.get_mut(&withdrawal_request.user) {
            let currency = withdrawal_request.address.currency();
            if let Some(cur_info) = user.currencies.get_mut(&currency) {
                cur_info
                    .withdrawal_requests
                    .insert(withdrawal_request_info.id, withdrawal_request);
                Ok(())
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
            WithdrawalRequestStatus::Completed{..} => {
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
            WithdrawalRequestStatus::NodeRejected{..} => {
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
                            return Ok(None)
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
                info.deposit_info.push(dep_address.address);
                Ok(())
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

        let txid = CurrencyTxId::BTC(BTCTxid{txid: tx.txid.0.to_string()});
        let res2 = if let Some(rid) = self.find_withdrawal_by_tx_id(txid){
            let reject = WithdrawalRejectInfo{
                id: rid,
                reason: "Tx canceled".to_owned(),
            };
            self.set_withdrawal_request_node_rejected(reject)
        } else {
            Ok(())
        };
        if res1.is_err() && res2.is_err() {
            res1
        } else {Ok(())}
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

    pub fn get_withdrawal_request(&self, id: WithdrawalRequestId) -> Option<WithdrawalRequest>{
        for (_, user) in self.users.iter() {
            for (_, info) in user.currencies.iter(){
                for (req_id, req) in info.withdrawal_requests.iter(){
                    if req_id.clone() == id {
                        return Some(req.clone())
                    }
                }
            }
        }
        None
    }

    pub fn set_withdrawal_request_completed(&mut self, withdrawal_confirmed_info: WithdrawCompleteInfo) -> Result<(), StateUpdateErr>{
        for (_, user) in self.users.iter_mut() {
            for (_, info) in user.currencies.iter_mut(){
                for (req_id, req) in info.withdrawal_requests.iter_mut(){
                    if req_id.clone() == withdrawal_confirmed_info.id {
                        let stat = WithdrawalRequestStatus::Completed { 
                            confirmed_at: withdrawal_confirmed_info.confirmed_at,
                            txid: withdrawal_confirmed_info.txid.clone(),
                            fee:  withdrawal_confirmed_info.fee.unwrap_or(0) };
                        req.status = stat;
                    }
                }
            }
        };
        Ok(())
    }

    pub fn set_withdrawal_request_node_rejected(&mut self, reject_info: WithdrawalRejectInfo) -> Result<(), StateUpdateErr> {
        for (_, user) in self.users.iter_mut() {
            for (_, info) in user.currencies.iter_mut(){
                for (req_id, req) in info.withdrawal_requests.iter_mut(){
                    if req_id.clone() == reject_info.id {
                        let stat = WithdrawalRequestStatus::NodeRejected { 
                            reason: reject_info.reason.clone()
                        };
                        req.status = stat;
                    }
                }
            }
        };
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
        let signup_info = SignupInfo {
            username: "Alice".to_owned(),
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
        let signup_info = SignupInfo {
            username: "Alice".to_owned(),
            auth: SignupAuth::Lightning,
        };
        let withdrawal_request_info = WithdrawalRequestInfo {
            id: Uuid::new_v4(),
            user: signup_info.username.clone(),
            address: CurrencyAddress::BTC(BtcAddress {
                addr: "bc1qpv8tczdsft9lmlz4nhz8058jdyl96velqqlwgj".to_owned(),
            }),
            amount: 1,
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
                rejections: vec![]
            }
        );
    }

    #[sqlx_database_tester::test(pool(variable = "pool", migrations = "./migrations"))]
    async fn test_new_withdrawal_request_decision_update() {
        let mut state = State::default();
        let signup_info = SignupInfo {
            username: "Alice".to_owned(),
            auth: SignupAuth::Lightning,
        };
        let withdrawal_request_info = WithdrawalRequestInfo {
            id: Uuid::new_v4(),
            user: signup_info.username.clone(),
            address: CurrencyAddress::BTC(BtcAddress {
                addr: "bc1qpv8tczdsft9lmlz4nhz8058jdyl96velqqlwgj".to_owned(),
            }),
            amount: 1,
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

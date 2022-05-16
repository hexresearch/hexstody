pub mod transaction;
pub mod user;
pub mod withdraw;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
pub use transaction::*;
pub use user::*;
use uuid::Uuid;
pub use withdraw::*;

use super::update::signup::{SignupInfo, UserId};
use super::update::withdrawal::WithdrawalRequestInfo;
use super::update::deposit::DepositAddress;
use super::update::{StateUpdate, UpdateBody,};
use hexstody_api::domain::Currency;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct State {
    /// All known users of the system.
    /// TODO: There is possible DDoS attack on signup of million of users.
    ///     We need to implement rate limits for it and auto cleanup of unused empty accounts.
    pub users: HashMap<UserId, UserInfo>,
    /// Tracks when the state was last updated
    pub last_changed: NaiveDateTime,
}

#[derive(Error, Debug, PartialEq)]
pub enum StateUpdateErr {
    #[error("User with ID {0} is already signed up")]
    UserAlreadyExists(UserId),
    #[error("User with ID {0} is not known")]
    CannotFoundUser(UserId),
    #[error("User {0} doesn't have currency {1}")]
    UserMissingCurrency(UserId, Currency),
}

impl State {
    pub fn new() -> Self {
        State {
            users: HashMap::new(),
            last_changed: Utc::now().naive_utc(),
        }
    }

    /// Apply an update event from persistent store
    pub fn apply_update(&mut self, update: StateUpdate) -> Result<(), StateUpdateErr> {
        match update.body {
            UpdateBody::Signup(info) => {
                self.with_signup(update.created, info)?;
                self.last_changed = update.created;
                Ok(())
            }
            UpdateBody::Snapshot(snaphsot) => {
                *self = snaphsot;
                self.last_changed = update.created;
                Ok(())
            }
            UpdateBody::NewWithdrawalRequest(withdrawal_request) => {
                self.with_new_withdrawal_request(update.created, withdrawal_request)?;
                self.last_changed = update.created;
                Ok(())
            }
            UpdateBody::DepositAddress(dep_address) => {
                self.with_deposit_address(dep_address)?;
                self.last_changed = update.created;
                Ok(())
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
        let request_id = Uuid::new_v4();
        let withdrawal_request: WithdrawalRequest =
            (timestamp, request_id, withdrawal_request_info).into();
        let user_id = &withdrawal_request.user;
        if let Some(user) = self.users.get_mut(user_id) {
            let currency = withdrawal_request.address.currency();
            if let Some(cur_info) = user.currencies.get_mut(&currency) {
                cur_info
                    .withdrawal_requests
                    .insert(request_id, withdrawal_request);
                Ok(())
            } else {
                Err(StateUpdateErr::UserMissingCurrency(
                    user_id.clone(),
                    currency,
                ))
            }
        } else {
            Err(StateUpdateErr::CannotFoundUser(user_id.clone()))
        }
    }

    /// Apply new withdrawal request update
    fn with_deposit_address(
        &mut self,
        dep_address: DepositAddress,
    ) -> Result<(), StateUpdateErr> {
        let user_id = &dep_address.user_id;
        if let Some(user) = self.users.get_mut(user_id) {
            let currency = dep_address.address.currency();
            if let Some(info) = user.currencies.get_mut(&currency) {
                info.deposit_info.push(dep_address.address);
                Ok(())
            } else {
                Err(StateUpdateErr::UserMissingCurrency(user_id.clone(), currency))
            }
        } else {
            Err(StateUpdateErr::CannotFoundUser(user_id.clone()))
        }
    }

    /// Take ordered chain of updates and collect the accumulated state.
    /// Order should be from the earliest to the latest.
    pub fn collect<I>(updates: I) -> Result<Self, StateUpdateErr>
    where
        I: IntoIterator<Item = StateUpdate>,
    {
        let mut state = State::new();
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
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queries::*;
    use crate::update::signup::{SignupAuth, SignupInfo};
    use crate::update::StateUpdate;
    use hexstody_api::domain::{BtcAddress, CurrencyAddress};

    #[sqlx_database_tester::test(pool(variable = "pool", migrations = "./migrations"))]
    async fn test_signup_update() {
        let mut state0 = State::default();
        let username = "aboba".to_owned();
        let upd = StateUpdate::new(UpdateBody::Signup(SignupInfo {
            username: username.clone(),
            auth: SignupAuth::Lightning,
        }));
        insert_update(&pool, upd.body.clone(), Some(upd.created))
            .await
            .unwrap();
        let created_at = upd.created;
        state0.apply_update(upd).unwrap();

        let state = query_state(&pool).await.unwrap();
        let expected_user = UserInfo::new(&username, SignupAuth::Lightning, created_at);
        let extracted_user = state.users.get(&username).cloned().map(|mut u| {
            u.created_at = created_at;
            u
        });
        assert_eq!(extracted_user, Some(expected_user));
    }

    #[sqlx_database_tester::test(pool(variable = "pool", migrations = "./migrations"))]
    async fn test_new_withdrawal_request_update() {
        let mut state0 = State::default();
        let username = "bob".to_owned();
        let amount: u64 = 1;
        let address = CurrencyAddress::BTC(BtcAddress(
            "bc1qpv8tczdsft9lmlz4nhz8058jdyl96velqqlwgj".to_owned(),
        ));
        let signup_upd = StateUpdate::new(UpdateBody::Signup(SignupInfo {
            username: username.clone(),
            auth: SignupAuth::Lightning,
        }));
        insert_update(&pool, signup_upd.body.clone(), Some(signup_upd.created))
            .await
            .unwrap();
        let upd = StateUpdate::new(UpdateBody::NewWithdrawalRequest(WithdrawalRequestInfo {
            user: username.clone(),
            address: address.clone(),
            amount,
        }));
        insert_update(&pool, upd.body.clone(), Some(upd.created))
            .await
            .unwrap();
        state0.apply_update(signup_upd).unwrap();
        state0.apply_update(upd).unwrap();
        let state = query_state(&pool).await.unwrap();
        let extracted_withdrawal_request = state
            .users
            .get(&username)
            .unwrap()
            .currencies
            .get(&Currency::BTC)
            .unwrap()
            .withdrawal_requests
            .iter()
            .next()
            .unwrap()
            .1;
        assert_eq!(extracted_withdrawal_request.user, username);
        assert_eq!(extracted_withdrawal_request.address, address);
        assert_eq!(extracted_withdrawal_request.amount, amount);
        assert_eq!(
            extracted_withdrawal_request.confrimtaion_status,
            WithdrawalRequestStatus::Confirmations(Vec::new())
        );
    }
}

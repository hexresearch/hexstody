use chrono::prelude::*;
use ecdsa::{Signature, VerifyingKey};
use p256::NistP256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use super::update::signup::{SignupAuth, SignupInfo, UserId};
use super::update::withdrawal::WithdrawalRequestInfo;
use super::update::{StateUpdate, UpdateBody};
use crate::domain::{Currency, CurrencyAddress};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct State {
    /// All known users of the system.
    /// TODO: There is possible DDoS attack on signup of million of users.
    ///     We need to implement rate limits for it and auto cleanup of unused empty accounts.
    pub users: HashMap<UserId, UserInfo>,
    /// Users can create withdrawal requests that in some cases require manual confirmation from operators
    pub withdrawal_requests: HashMap<WithdrawalRequestId, WithdrawalRequest>,
    /// Tracks when the state was last updated
    pub last_changed: NaiveDateTime,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    /// It is unique user ID whithin the system. It is either email address or hex encoded LNAuth public key.
    pub username: UserId,
    /// Contains additional info that required to authentificated user in future.
    pub auth: SignupAuth,
    /// When the user was created
    pub created_at: NaiveDateTime,
    /// Required information for making deposit for the user in different currencies.
    pub deposit_info: HashMap<Currency, Vec<CurrencyAddress>>,
}

impl From<(NaiveDateTime, SignupInfo)> for UserInfo {
    fn from(value: (NaiveDateTime, SignupInfo)) -> Self {
        UserInfo {
            username: value.1.username,
            auth: value.1.auth,
            created_at: value.0,
            deposit_info: HashMap::new(),
        }
    }
}

/// It is unique withdrawal request ID whithin the system.
pub type WithdrawalRequestId = Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WithdrawalRequest {
    /// Request ID
    pub id: WithdrawalRequestId,
    /// User which initiated request
    pub user: UserId,
    /// Receiving address
    pub address: CurrencyAddress,
    /// When the request was created
    pub created_at: NaiveDateTime,
    /// Amount of tokens to transfer
    pub amount: u64,
    /// Some request require manual confirmation
    pub confrimtaion_status: WithdrawalRequestStatus,
}

impl From<(NaiveDateTime, WithdrawalRequestId, WithdrawalRequestInfo)> for WithdrawalRequest {
    fn from(value: (NaiveDateTime, WithdrawalRequestId, WithdrawalRequestInfo)) -> Self {
        WithdrawalRequest {
            id: value.1,
            user: value.2.user,
            address: value.2.address,
            created_at: value.0,
            amount: value.2.amount,
            confrimtaion_status: WithdrawalRequestStatus::Confirmations(Vec::new()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum WithdrawalRequestStatus {
    /// This request doesn't require manual confirmation
    NoConfirmationRequired,
    /// Vector of confirmations received from operators
    Confirmations(Vec<(VerifyingKey<NistP256>, Signature<NistP256>)>),
}

#[derive(Error, Debug, PartialEq)]
pub enum StateUpdateErr {
    #[error("User with ID {0} is already signed up")]
    UserAlreadyExists(String),
}

impl State {
    pub fn new() -> Self {
        State {
            users: HashMap::new(),
            last_changed: Utc::now().naive_utc(),
            withdrawal_requests: HashMap::new(),
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
        self.withdrawal_requests
            .insert(request_id, withdrawal_request);

        Ok(())
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
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{BtcAddress, CurrencyAddress};
    use crate::queries::*;
    use crate::update::signup::SignupInfo;
    use crate::update::StateUpdate;

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
        let expected_user = UserInfo {
            username: username.clone(),
            auth: SignupAuth::Lightning,
            deposit_info: HashMap::new(),
            created_at,
        };
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
        let upd = StateUpdate::new(UpdateBody::NewWithdrawalRequest(WithdrawalRequestInfo {
            user: username.clone(),
            address: address.clone(),
            amount,
        }));
        insert_update(&pool, upd.body.clone(), Some(upd.created))
            .await
            .unwrap();
        state0.apply_update(upd).unwrap();
        let state = query_state(&pool).await.unwrap();
        let extracted_withdrawal_request = state.withdrawal_requests.iter().next().unwrap().1;
        assert_eq!(extracted_withdrawal_request.user, username);
        assert_eq!(extracted_withdrawal_request.address, address);
        assert_eq!(extracted_withdrawal_request.amount, amount);
        assert_eq!(
            extracted_withdrawal_request.confrimtaion_status,
            WithdrawalRequestStatus::Confirmations(Vec::new())
        );
    }
}

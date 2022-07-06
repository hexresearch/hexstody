pub mod btc;
pub mod deposit;
pub mod signup;
pub mod withdrawal;
pub mod results;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

use self::btc::{BestBtcBlock, BtcTxCancel};
use self::deposit::DepositAddress;
use self::signup::SignupInfo;
use self::withdrawal::{WithdrawalRequestDecisionInfo, WithdrawalRequestInfo};
use super::state::transaction::BtcTransaction;
use super::state::State;

/// All database updates are collected to a single table that
/// allows to reconstruct current state of the system by replaying
/// all events until required timestamp.
#[derive(Debug, PartialEq, Clone)]
pub struct StateUpdate {
    pub created: NaiveDateTime,
    pub body: UpdateBody,
}

impl StateUpdate {
    pub fn new(body: UpdateBody) -> Self {
        StateUpdate {
            created: Utc::now().naive_utc(),
            body,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum UpdateBody {
    /// Create new account for user
    Signup(SignupInfo),
    /// Caching current state to database for speeding startup time
    Snapshot(State),
    /// Create new withdrawal request
    CreateWithdrawalRequest(WithdrawalRequestInfo),
    /// New operator's decision for the withdrawal request
    WithdrawalRequestDecision(WithdrawalRequestDecisionInfo),
    /// Register new deposit address for user
    DepositAddress(DepositAddress),
    /// New best block for BTC
    BestBtcBlock(BestBtcBlock),
    /// Update state of BTC transaction
    UpdateBtcTx(BtcTransaction),
    /// Cancel BTC transaction
    CancelBtcTx(BtcTxCancel),
}

impl UpdateBody {
    pub fn tag(&self) -> UpdateTag {
        match self {
            UpdateBody::Signup(_) => UpdateTag::Signup,
            UpdateBody::Snapshot(_) => UpdateTag::Snapshot,
            UpdateBody::CreateWithdrawalRequest(_) => UpdateTag::CreateWithdrawalRequest,
            UpdateBody::WithdrawalRequestDecision(_) => UpdateTag::WithdrawalRequestDecision,
            UpdateBody::DepositAddress(_) => UpdateTag::DepositAddress,
            UpdateBody::BestBtcBlock(_) => UpdateTag::BestBtcBlock,
            UpdateBody::UpdateBtcTx(_) => UpdateTag::UpdateBtcTx,
            UpdateBody::CancelBtcTx(_) => UpdateTag::CancelBtcTx,
        }
    }

    pub fn json(&self) -> serde_json::Result<serde_json::Value> {
        match self {
            UpdateBody::Signup(v) => serde_json::to_value(v),
            UpdateBody::Snapshot(v) => serde_json::to_value(v),
            UpdateBody::CreateWithdrawalRequest(v) => serde_json::to_value(v),
            UpdateBody::WithdrawalRequestDecision(v) => serde_json::to_value(v),
            UpdateBody::DepositAddress(v) => serde_json::to_value(v),
            UpdateBody::BestBtcBlock(v) => serde_json::to_value(v),
            UpdateBody::UpdateBtcTx(v) => serde_json::to_value(v),
            UpdateBody::CancelBtcTx(v) => serde_json::to_value(v),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum UpdateTag {
    Signup,
    Snapshot,
    CreateWithdrawalRequest,
    WithdrawalRequestDecision,
    DepositAddress,
    BestBtcBlock,
    UpdateBtcTx,
    CancelBtcTx,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct UnknownUpdateTag(String);

impl std::error::Error for UnknownUpdateTag {}

impl fmt::Display for UnknownUpdateTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Given UpdateTag '{}' is unknown", self.0)
    }
}

impl fmt::Display for UpdateTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateTag::Signup => write!(f, "signup"),
            UpdateTag::Snapshot => write!(f, "snapshot"),
            UpdateTag::CreateWithdrawalRequest => write!(f, "withdrawal request"),
            UpdateTag::WithdrawalRequestDecision => write!(f, "withdrawal request decision"),
            UpdateTag::DepositAddress => write!(f, "deposit address"),
            UpdateTag::BestBtcBlock => write!(f, "best btc block"),
            UpdateTag::UpdateBtcTx => write!(f, "update btc tx"),
            UpdateTag::CancelBtcTx => write!(f, "cancel btc tx"),
        }
    }
}

impl FromStr for UpdateTag {
    type Err = UnknownUpdateTag;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "signup" => Ok(UpdateTag::Signup),
            "snapshot" => Ok(UpdateTag::Snapshot),
            "withdrawal request" => Ok(UpdateTag::CreateWithdrawalRequest),
            "withdrawal request decision" => Ok(UpdateTag::WithdrawalRequestDecision),
            "deposit address" => Ok(UpdateTag::DepositAddress),
            "best btc block" => Ok(UpdateTag::BestBtcBlock),
            "update btc tx" => Ok(UpdateTag::UpdateBtcTx),
            "cancel btc tx" => Ok(UpdateTag::CancelBtcTx),
            _ => Err(UnknownUpdateTag(s.to_owned())),
        }
    }
}

#[derive(Error, Debug)]
pub enum UpdateBodyError {
    #[error("Unknown update tag: {0}")]
    UnknownTag(#[from] UnknownUpdateTag),
    #[error("Failed to deserialize body with version {0} and tag {1}: {2}. Body: {3}")]
    Deserialize(u16, UpdateTag, serde_json::Error, serde_json::Value),
    #[error("Unknown version tag: {0}")]
    UnexpectedVersion(u16),
}

pub const CURRENT_BODY_VERSION: u16 = 0;

impl UpdateTag {
    pub fn from_tag(
        tag: &str,
        version: u16,
        value: serde_json::Value,
    ) -> Result<UpdateBody, UpdateBodyError> {
        let tag = <UpdateTag as FromStr>::from_str(tag)?;
        if version != CURRENT_BODY_VERSION {
            return Err(UpdateBodyError::UnexpectedVersion(version));
        }
        tag.deserialize(value.clone())
            .map_err(|e| UpdateBodyError::Deserialize(version, tag, e, value))
    }

    pub fn deserialize(&self, value: serde_json::Value) -> Result<UpdateBody, serde_json::Error> {
        match self {
            UpdateTag::Signup => Ok(UpdateBody::Signup(serde_json::from_value(value)?)),
            UpdateTag::Snapshot => Ok(UpdateBody::Snapshot(serde_json::from_value(value)?)),
            UpdateTag::CreateWithdrawalRequest => Ok(UpdateBody::CreateWithdrawalRequest(
                serde_json::from_value(value)?,
            )),
            UpdateTag::WithdrawalRequestDecision => Ok(UpdateBody::WithdrawalRequestDecision(
                serde_json::from_value(value)?,
            )),
            UpdateTag::DepositAddress => {
                Ok(UpdateBody::DepositAddress(serde_json::from_value(value)?))
            }
            UpdateTag::BestBtcBlock => Ok(UpdateBody::BestBtcBlock(serde_json::from_value(value)?)),
            UpdateTag::UpdateBtcTx => Ok(UpdateBody::UpdateBtcTx(serde_json::from_value(value)?)),
            UpdateTag::CancelBtcTx => Ok(UpdateBody::CancelBtcTx(serde_json::from_value(value)?)),
        }
    }
}

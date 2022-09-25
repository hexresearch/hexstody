pub mod btc;
pub mod deposit;
pub mod signup;
pub mod withdrawal;
pub mod results;
pub mod misc;
pub mod limit;

use chrono::prelude::*;
use hexstody_api::domain::CurrencyAddress;
use hexstody_api::types::LimitSpan;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

use crate::state::exchange::{ExchangeOrderUpd, ExchangeDecision};

use self::btc::{BestBtcBlock, BtcTxCancel};
use self::deposit::DepositAddress;
use self::limit::{LimitChangeUpd, LimitCancelData, LimitChangeDecision};
use self::signup::SignupInfo;
use self::withdrawal::{WithdrawalRequestDecisionInfo, WithdrawalRequestInfo, WithdrawCompleteInfo, WithdrawalRejectInfo};
use self::misc::{InviteRec, TokenUpdate, SetLanguage, ConfigUpdateData, PasswordChangeUpd, SetPublicKey};
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
    /// Set withdraw request to confirmed
    WithdrawalRequestComplete(WithdrawCompleteInfo),
    /// Withdrawal request rejected by the node
    WithdrawalRequestNodeRejected(WithdrawalRejectInfo),
    /// Register new deposit address for user
    DepositAddress(DepositAddress),
    /// New best block for BTC
    BestBtcBlock(BestBtcBlock),
    /// Update state of BTC transaction
    UpdateBtcTx(BtcTransaction),
    /// Cancel BTC transaction
    CancelBtcTx(BtcTxCancel),
    /// Update token list
    UpdateTokens(TokenUpdate),
    /// Generate invite
    GenInvite(InviteRec),
    /// Register limits change
    LimitsChangeRequest(LimitChangeUpd),
    /// Cancel limit change request
    CancelLimitChange(LimitCancelData),
    /// Limit change decision
    LimitChangeDecision(LimitChangeDecision),
    /// Clear limits by span
    ClearLimits(LimitSpan),
    /// Set language
    SetLanguage(SetLanguage),
    /// Update user's config
    ConfigUpdate(ConfigUpdateData),
    /// Change user's password
    PasswordChange(PasswordChangeUpd),
    /// Set user's public key
    SetPublicKey(SetPublicKey),
    /// Request an exchange from operators
    ExchangeRequest(ExchangeOrderUpd),
    /// Exchange decision
    ExchangeDecision(ExchangeDecision),
    /// Set up exchange deposit address
    ExchangeAddress(CurrencyAddress)
}

impl UpdateBody {
    pub fn tag(&self) -> UpdateTag {
        match self {
            UpdateBody::Signup(_) => UpdateTag::Signup,
            UpdateBody::Snapshot(_) => UpdateTag::Snapshot,
            UpdateBody::CreateWithdrawalRequest(_) => UpdateTag::CreateWithdrawalRequest,
            UpdateBody::WithdrawalRequestDecision(_) => UpdateTag::WithdrawalRequestDecision,
            UpdateBody::WithdrawalRequestComplete(_) => UpdateTag::WithdrawalRequestConfirm,
            UpdateBody::WithdrawalRequestNodeRejected(_) => UpdateTag::WithdrawalRequestNodeRejected,
            UpdateBody::DepositAddress(_) => UpdateTag::DepositAddress,
            UpdateBody::BestBtcBlock(_) => UpdateTag::BestBtcBlock,
            UpdateBody::UpdateBtcTx(_) => UpdateTag::UpdateBtcTx,
            UpdateBody::CancelBtcTx(_) => UpdateTag::CancelBtcTx,
            UpdateBody::UpdateTokens(_) => UpdateTag::UpdateTokens,
            UpdateBody::GenInvite(_) => UpdateTag::GenInvite,
            UpdateBody::LimitsChangeRequest(_) => UpdateTag::LimitsChangeRequest,
            UpdateBody::CancelLimitChange(_) => UpdateTag::CancelLimitChange,
            UpdateBody::LimitChangeDecision(_) => UpdateTag::LimitChangeDecision,
            UpdateBody::ClearLimits(_) => UpdateTag::ClearLimits,
            UpdateBody::SetLanguage(_) => UpdateTag::SetLanguage,
            UpdateBody::ConfigUpdate(_) => UpdateTag::ConfigUpdate,
            UpdateBody::PasswordChange(_) => UpdateTag::PasswordChange,
            UpdateBody::SetPublicKey(_) => UpdateTag::SetPublicKey,
            UpdateBody::ExchangeRequest(_) => UpdateTag::ExchangeRequest,
            UpdateBody::ExchangeDecision(_) => UpdateTag::ExchangeDecision,
            UpdateBody::ExchangeAddress(_) => UpdateTag::ExchangeAddress,
        }
    }

    pub fn json(&self) -> serde_json::Result<serde_json::Value> {
        match self {
            UpdateBody::Signup(v) => serde_json::to_value(v),
            UpdateBody::Snapshot(v) => serde_json::to_value(v),
            UpdateBody::CreateWithdrawalRequest(v) => serde_json::to_value(v),
            UpdateBody::WithdrawalRequestDecision(v) => serde_json::to_value(v),
            UpdateBody::WithdrawalRequestComplete(v) => serde_json::to_value(v),
            UpdateBody::WithdrawalRequestNodeRejected(v) => serde_json::to_value(v),
            UpdateBody::DepositAddress(v) => serde_json::to_value(v),
            UpdateBody::BestBtcBlock(v) => serde_json::to_value(v),
            UpdateBody::UpdateBtcTx(v) => serde_json::to_value(v),
            UpdateBody::CancelBtcTx(v) => serde_json::to_value(v),
            UpdateBody::UpdateTokens(v) => serde_json::to_value(v),
            UpdateBody::GenInvite(v) => serde_json::to_value(v),
            UpdateBody::LimitsChangeRequest(v) => serde_json::to_value(v),
            UpdateBody::CancelLimitChange(v) => serde_json::to_value(v),
            UpdateBody::LimitChangeDecision(v) => serde_json::to_value(v),
            UpdateBody::ClearLimits(v) => serde_json::to_value(v),
            UpdateBody::SetLanguage(v) => serde_json::to_value(v),
            UpdateBody::ConfigUpdate(v) => serde_json::to_value(v),
            UpdateBody::PasswordChange(v) => serde_json::to_value(v),
            UpdateBody::SetPublicKey(v) => serde_json::to_value(v),
            UpdateBody::ExchangeRequest(v) => serde_json::to_value(v),
            UpdateBody::ExchangeDecision(v) => serde_json::to_value(v),
            UpdateBody::ExchangeAddress(v) => serde_json::to_value(v),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum UpdateTag {
    Signup,
    Snapshot,
    CreateWithdrawalRequest,
    WithdrawalRequestDecision,
    WithdrawalRequestConfirm,
    WithdrawalRequestNodeRejected,
    DepositAddress,
    BestBtcBlock,
    UpdateBtcTx,
    CancelBtcTx,
    UpdateTokens,
    GenInvite,
    LimitsChangeRequest,
    CancelLimitChange,
    LimitChangeDecision,
    ClearLimits,
    SetLanguage,
    ConfigUpdate,
    PasswordChange,
    SetPublicKey,
    ExchangeRequest,
    ExchangeDecision,
    ExchangeAddress,
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
            UpdateTag::WithdrawalRequestConfirm => write!(f, "withdrawal request confirm"),
            UpdateTag::WithdrawalRequestNodeRejected => write!(f, "withdrawal request node rejected"),
            UpdateTag::DepositAddress => write!(f, "deposit address"),
            UpdateTag::BestBtcBlock => write!(f, "best btc block"),
            UpdateTag::UpdateBtcTx => write!(f, "update btc tx"),
            UpdateTag::CancelBtcTx => write!(f, "cancel btc tx"),
            UpdateTag::UpdateTokens => write!(f, "update tokens"),
            UpdateTag::GenInvite => write!(f, "gen invite"),
            UpdateTag::LimitsChangeRequest => write!(f, "limits change req"),
            UpdateTag::CancelLimitChange => write!(f, "cancel limits change"),
            UpdateTag::LimitChangeDecision => write!(f, "limit change decision"),
            UpdateTag::ClearLimits => write!(f, "clear limits"),
            UpdateTag::SetLanguage => write!(f, "set language"),
            UpdateTag::ConfigUpdate => write!(f, "user config update"),
            UpdateTag::PasswordChange => write!(f, "password change"),
            UpdateTag::SetPublicKey => write!(f, "set public key"),
            UpdateTag::ExchangeRequest => write!(f, "exchange request"),
            UpdateTag::ExchangeDecision => write!(f, "exchange decision"),
            UpdateTag::ExchangeAddress => write!(f, "exchange address"),
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
            "withdrawal request confirm" => Ok(UpdateTag::WithdrawalRequestConfirm),
            "withdrawal request node rejected" => Ok(UpdateTag::WithdrawalRequestNodeRejected),
            "deposit address" => Ok(UpdateTag::DepositAddress),
            "best btc block" => Ok(UpdateTag::BestBtcBlock),
            "update btc tx" => Ok(UpdateTag::UpdateBtcTx),
            "cancel btc tx" => Ok(UpdateTag::CancelBtcTx),
            "update tokens" => Ok(UpdateTag::UpdateTokens),
            "gen invite" => Ok(UpdateTag::GenInvite),
            "limits change req" => Ok(UpdateTag::LimitsChangeRequest),
            "cancel limits change" => Ok(UpdateTag::CancelLimitChange),
            "limit change decision" => Ok(UpdateTag::LimitChangeDecision),
            "clear limits" => Ok(UpdateTag::ClearLimits),
            "set language" => Ok(UpdateTag::SetLanguage),
            "user config update" => Ok(UpdateTag::ConfigUpdate),
            "password change" => Ok(UpdateTag::PasswordChange),
            "set public key" => Ok(UpdateTag::SetPublicKey),
            "exchange request" => Ok(UpdateTag::ExchangeRequest),
            "exchange decision" => Ok(UpdateTag::ExchangeDecision),
            "exchange address" => Ok(UpdateTag::ExchangeAddress),
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
            UpdateTag::WithdrawalRequestConfirm => Ok(UpdateBody::WithdrawalRequestComplete(
                serde_json::from_value(value)?,
            )),
            UpdateTag::WithdrawalRequestNodeRejected => Ok(UpdateBody::WithdrawalRequestNodeRejected(serde_json::from_value(value)?)),
            UpdateTag::DepositAddress => {
                Ok(UpdateBody::DepositAddress(serde_json::from_value(value)?))
            }
            UpdateTag::BestBtcBlock => Ok(UpdateBody::BestBtcBlock(serde_json::from_value(value)?)),
            UpdateTag::UpdateBtcTx => Ok(UpdateBody::UpdateBtcTx(serde_json::from_value(value)?)),
            UpdateTag::CancelBtcTx => Ok(UpdateBody::CancelBtcTx(serde_json::from_value(value)?)),
            UpdateTag::UpdateTokens => Ok(UpdateBody::UpdateTokens(serde_json::from_value(value)?)),
            UpdateTag::GenInvite => Ok(UpdateBody::GenInvite(serde_json::from_value(value)?)),
            UpdateTag::LimitsChangeRequest => Ok(UpdateBody::LimitsChangeRequest(serde_json::from_value(value)?)),
            UpdateTag::CancelLimitChange => Ok(UpdateBody::CancelLimitChange(serde_json::from_value(value)?)),
            UpdateTag::LimitChangeDecision => Ok(UpdateBody::LimitChangeDecision(serde_json::from_value(value)?)),
            UpdateTag::ClearLimits => Ok(UpdateBody::ClearLimits(serde_json::from_value(value)?)),
            UpdateTag::SetLanguage => Ok(UpdateBody::SetLanguage(serde_json::from_value(value)?)),
            UpdateTag::ConfigUpdate => Ok(UpdateBody::ConfigUpdate(serde_json::from_value(value)?)),
            UpdateTag::PasswordChange => Ok(UpdateBody::PasswordChange(serde_json::from_value(value)?)),
            UpdateTag::SetPublicKey => Ok(UpdateBody::SetPublicKey(serde_json::from_value(value)?)),
            UpdateTag::ExchangeRequest => Ok(UpdateBody::ExchangeRequest(serde_json::from_value(value)?)),
            UpdateTag::ExchangeDecision => Ok(UpdateBody::ExchangeDecision(serde_json::from_value(value)?)),
            UpdateTag::ExchangeAddress => Ok(UpdateBody::ExchangeAddress(serde_json::from_value(value)?)),
        }
    }
}

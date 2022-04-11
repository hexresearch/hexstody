pub mod signup;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

use super::state::State;
use self::signup::SignupInfo;

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
}

impl UpdateBody {
    pub fn tag(&self) -> UpdateTag {
        match self {
            UpdateBody::Signup(_) => UpdateTag::Signup,
            UpdateBody::Snapshot(_) => UpdateTag::Snapshot,
        }
    }

    pub fn json(&self) -> serde_json::Result<serde_json::Value> {
        match self {
            UpdateBody::Signup(v) => serde_json::to_value(v),
            UpdateBody::Snapshot(v) => serde_json::to_value(v),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum UpdateTag {
    Signup,
    Snapshot,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct UnknownUpdateTag(String);

impl std::error::Error for UnknownUpdateTag {}

impl fmt::Display for UnknownUpdateTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Given UpdateTag '{}' is unknown, valid are: Htlc, Snapshot",
            self.0
        )
    }
}

impl fmt::Display for UpdateTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateTag::Signup => write!(f, "signup"),
            UpdateTag::Snapshot => write!(f, "snapshot"),
        }
    }
}

impl FromStr for UpdateTag {
    type Err = UnknownUpdateTag;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "signup" => Ok(UpdateTag::Signup),
            "snapshot" => Ok(UpdateTag::Snapshot),
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
        }
    }
}
use serde::{Deserialize, Serialize};

/// It is unique user ID whithin the system. It is either email address or hex encoded LNAuth public key.
pub type UserId = String;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SignupInfo {
    /// It is unique user ID whithin the system. It is either email address or hex encoded LNAuth public key.
    pub username: UserId,
    /// Contains additional info that required to authentificated user in future.
    pub auth: SignupAuth,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum SignupAuth {
    /// User hashed and salted password
    Password(String),
    /// Lightning users should provide signature for each session, so there
    /// is no need to store additional information in persistent storage.
    Lightning,
}
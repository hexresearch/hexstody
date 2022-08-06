use serde::{Deserialize, Serialize};
use super::signup::UserId;

use hexstody_api::domain::Erc20Token;
use hexstody_api::types::Invite;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum TokenAction {
    Enable,
    Disable
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TokenUpdate{
    pub user: UserId,
    pub token: Erc20Token,
    pub action: TokenAction
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct InviteRec{
    /// The invite in question
    pub invite: Invite,
    /// String rep of public key of the operator, who generated an invite
    pub invitor: String,
    /// Invite label
    pub label: String
}
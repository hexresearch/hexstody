use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::signup::UserId;

use hexstody_api::domain::{Erc20Token, Currency};
use hexstody_api::types::{Invite, Limit};

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct LimitCancelData{
    pub id: Uuid,
    pub user: String,
    pub currency: Currency
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct LimitChangeUpd{
    pub user: String,
    pub currency: Currency,
    pub limit: Limit
}
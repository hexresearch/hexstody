use serde::{Serialize, Deserialize};

use crate::state::withdraw::WithdrawalRequestId;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum UpdateResult {
    WithdrawConfirmed(WithdrawalRequestId),
}
use serde::{Serialize, Deserialize};

use crate::state::withdraw::{WithdrawalRequestId, WithdrawalRequest};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum UpdateResult {
    WithdrawConfirmed(WithdrawalRequestId),
    WithdrawalUnderlimit(WithdrawalRequest)
}
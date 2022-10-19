use hexstody_api::{domain::{Currency, CurrencyAddress, Erc20Token}, error::HexstodyError};
use p256::PublicKey;
use thiserror::Error;
use uuid::Uuid;

use crate::{update::signup::UserId, state::withdraw::WithdrawalRequestId};

#[derive(Error, Debug, PartialEq)]
pub enum StateUpdateErr {
    #[error("User with ID {0} is already signed up")]
    UserAlreadyExists(UserId),
    #[error("User with ID {0} is not known")]
    UserNotFound(UserId),
    #[error("User {0} doesn't have currency {1}")]
    UserMissingCurrency(UserId, Currency),
    #[error("Deposit address {1} is already allocated for user {0}")]
    DepositAddressAlreadyAllocated(UserId, CurrencyAddress),
    #[error("User {0} doesn't have withdrawal request {1}")]
    WithdrawalRequestNotFound(UserId, WithdrawalRequestId),
    #[error("Withdrawal request {0} is already confirmed by {}", .1.to_string())]
    WithdrawalRequestAlreadyConfirmedByThisKey(WithdrawalRequestId, PublicKey),
    #[error("Withdrawal request {0} is already rejected by {}", .1.to_string())]
    WithdrawalRequestAlreadyRejectedByThisKey(WithdrawalRequestId, PublicKey),
    #[error("Withdrawal request {0} is already confirmed")]
    WithdrawalRequestAlreadyConfirmed(WithdrawalRequestId),
    #[error("Withdrawal request {0} is already rejected")]
    WithdrawalRequestAlreadyRejected(WithdrawalRequestId),
    #[error("{0} is already enabled")]
    TokenAlreadyEnabled(Erc20Token),
    #[error("{0} is already disabled")]
    TokenAlreadyDisabled(Erc20Token),
    #[error("{0} has non-zero balance. Can not disable")]
    TokenNonZeroBalance(Erc20Token),
    #[error("Failed to enable token {0} from {1}")]
    TokenEnableFail(Erc20Token, UserId),
    #[error("Invite already exist")]
    InviteAlreadyExist,
    #[error("Invite is not valid")]
    InviteNotFound,
    #[error("Limit request does not exist")]
    LimitChangeNotFound,
    #[error("Limit request already signed by the operator")]
    LimitAlreadySigned,
    #[error("Limit request already confirmed and finalized")]
    LimitAlreadyConfirmed,
    #[error("Limit request already rejected")]
    LimitAlreadyRejected,
    #[error("The spending is over the limit")]
    LimitOverflow,
    #[error("User {0} doesn't have enough of currency {1}")]
    InsufficientFunds(UserId, Currency),
    #[error("User {0} doesn't have outstanding exchange request for {1}")]
    UserMissingExchange(String, Currency),
    #[error("Exchange request already signed by the operator")]
    ExchangeAlreadySigned,
    #[error("Exchange request already confirmed and finalized")]
    ExchangeAlreadyConfirmed,
    #[error("Exchange request already rejected")]
    ExchangeAlreadyRejected,
    #[error("Unknown currency: {0}")]
    UnknownCurrency(String),
    #[error("Invoice {} for user {1} not found", .0.to_string())]
    InvoiceNotFound(Uuid, String),
    #[error("Generic error: {0}")]
    GenericError(String)
}

impl HexstodyError for StateUpdateErr {
    fn subtype() -> &'static str {
        "hexstody-db:state"
    }

    fn code(&self) -> u16 {
        match self {
            StateUpdateErr::UserAlreadyExists(_) => 0,
            StateUpdateErr::UserNotFound(_) => 1,
            StateUpdateErr::UserMissingCurrency(_, _) => 2,
            StateUpdateErr::DepositAddressAlreadyAllocated(_, _) => 3,
            StateUpdateErr::WithdrawalRequestNotFound(_, _) => 4,
            StateUpdateErr::WithdrawalRequestAlreadyConfirmedByThisKey(_, _) => 5,
            StateUpdateErr::WithdrawalRequestAlreadyRejectedByThisKey(_, _) => 6,
            StateUpdateErr::WithdrawalRequestAlreadyConfirmed(_) => 7,
            StateUpdateErr::WithdrawalRequestAlreadyRejected(_) => 8,
            StateUpdateErr::TokenAlreadyEnabled(_) => 9,
            StateUpdateErr::TokenAlreadyDisabled(_) => 10,
            StateUpdateErr::TokenNonZeroBalance(_) => 11,
            StateUpdateErr::TokenEnableFail(_, _) => 12,
            StateUpdateErr::InviteAlreadyExist => 13,
            StateUpdateErr::InviteNotFound => 14,
            StateUpdateErr::LimitChangeNotFound => 15,
            StateUpdateErr::LimitAlreadySigned => 16,
            StateUpdateErr::LimitAlreadyConfirmed => 17,
            StateUpdateErr::LimitAlreadyRejected => 18,
            StateUpdateErr::LimitOverflow => 19,
            StateUpdateErr::InsufficientFunds(_, _) => 20,
            StateUpdateErr::UserMissingExchange(_, _) => 21,
            StateUpdateErr::ExchangeAlreadySigned => 22,
            StateUpdateErr::ExchangeAlreadyConfirmed => 23,
            StateUpdateErr::ExchangeAlreadyRejected => 24,
            StateUpdateErr::UnknownCurrency(_) => 25,
            StateUpdateErr::InvoiceNotFound(_,_) => 26,
            StateUpdateErr::GenericError(_) => 27,
        }
    }

    fn status(&self) -> u16 {
        500
    }
}
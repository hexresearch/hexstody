use std::collections::HashMap;

use hexstody_api::{domain::{Currency, CurrencyAddress}, types::{ExchangeStatus, SignatureData, ExchangeConfirmationData}};
use p256::{ecdsa::Signature, PublicKey};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::update::btc::BtcTxCancel;

use super::transaction::{BtcTransaction, Transaction, SameBtcTx};

pub type ExchangeOrderId = Uuid;

/// Used both in UpdateBody and in exchange storage, since it's slimmed down and has only necessary fields
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ExchangeOrderUpd {
    pub id: ExchangeOrderId,
    pub user: String,
    pub currency_from: Currency,
    pub currency_to: Currency,
    pub amount_from: u64,
    pub amount_to: u64,
    pub created_at: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ExchangeOrder {
    pub id: ExchangeOrderId,
    pub user: String,
    pub currency_from: Currency,
    pub currency_to: Currency,
    pub amount_from: u64,
    pub amount_to: u64,
    pub status: ExchangeStatus,
    pub created_at: String,
    pub confirmations: Vec<SignatureData>,
    pub rejections: Vec<SignatureData>
}

impl ExchangeOrder {
    pub fn is_finalized(&self) -> bool {
        matches!(self.status, ExchangeStatus::Completed)
    }
    pub fn is_rejected(&self) -> bool {
        matches!(self.status, ExchangeStatus::Rejected)
    }
    pub fn is_pending(&self) -> bool {
        matches!(self.status, ExchangeStatus::InProgress {..})
    }
    pub fn has_confirmed(&self, pubkey: PublicKey) -> bool{
        self.confirmations.iter().any(|sd| sd.public_key == pubkey)
    }
    pub fn has_rejected(&self, pubkey: PublicKey) -> bool{
        self.rejections.iter().any(|sd| sd.public_key == pubkey)
    }

    pub fn into_exchange_upd(&self) -> ExchangeOrderUpd {
        let ExchangeOrder { id, user, currency_from, currency_to, amount_from, amount_to, created_at, .. } = self.clone();
        ExchangeOrderUpd { user, id, currency_from, currency_to, amount_from, amount_to, created_at }
    }
}

impl From<ExchangeOrder> for hexstody_api::types::ExchangeOrder{
    fn from(eo: ExchangeOrder) -> Self {
        let ExchangeOrder { id, user, currency_from, currency_to, amount_from, amount_to, status, created_at, .. } = eo;
        hexstody_api::types::ExchangeOrder { user, id, currency_from, currency_to, amount_from, amount_to, status, created_at }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ExchangeDecisionType {
    Confirm,
    Reject
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ExchangeDecision {
    /// User who initiated an exchange
    pub user: String,
    /// Exchange id
    pub id: Uuid,
    /// Currency to exchange from
    pub currency_from: Currency,
    /// Currency to exchange to
    pub currency_to: Currency,
    /// Amount to exchange from
    pub amount_from: u64,
    /// Amount to exchange to
    pub amount_to: u64,
    /// API URL wich was used to send the decision
    pub url: String,
    /// Operator's digital signature
    pub signature: Signature,
    /// Nonce that was generated during decision
    pub nonce: u64,
    /// Operator's public key corresponding to the signing private key
    pub public_key: PublicKey,
    /// Decision type: confirm or reject
    pub decision: ExchangeDecisionType,
}

impl
    From<(
        ExchangeConfirmationData,
        SignatureData,
        ExchangeDecisionType,
        String,
    )> for ExchangeDecision
{
    fn from(
        value: (
            ExchangeConfirmationData,
            SignatureData,
            ExchangeDecisionType,
            String,
        ),
    ) -> ExchangeDecision {
        ExchangeDecision {
            user: value.0.user,
            id: value.0.id,
            currency_from: value.0.currency_from,
            currency_to: value.0.currency_to,
            amount_from: value.0.amount_from,
            amount_to: value.0.amount_to,
            url: value.3,
            signature: value.1.signature,
            nonce: value.1.nonce,
            public_key: value.1.public_key,
            decision: value.2,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ExchangeState{
    /// Slimmed down exchange orders. We keep here only completed orders 
    pub exchanges: HashMap<ExchangeOrderId, ExchangeOrderUpd>,
    /// Deposit addresses for exchange account
    pub addresses: HashMap<Currency, CurrencyAddress>,
    /// External deposits to exchange account
    pub deposits: Vec<Transaction>,
    /// We keep running balance, as to not calculate it each time
    pub balances: HashMap<Currency, i64>
}

impl ExchangeState {
    pub fn new() -> ExchangeState {
        ExchangeState { 
            exchanges: HashMap::new(),
            addresses: HashMap::new(),
            deposits: Vec::new(),
            balances: Currency::supported().iter().map(|c| (c.clone(),0)).collect()
        }
    }

    pub fn process_order(&mut self, order: ExchangeOrderUpd) {
        self.exchanges.insert(order.id.clone(), order.clone());
        self.balances.entry(order.currency_from).and_modify(|v| *v += order.amount_from as i64).or_insert(order.amount_from as i64);
        self.balances.entry(order.currency_to).and_modify(|v| *v -= order.amount_to as i64).or_insert(order.amount_to as i64);
    }

    pub fn process_incoming_btc_tx(&mut self, upd_tx: BtcTransaction) {
        if upd_tx.amount >= 0 {
            for tx in self.deposits.iter_mut() {
                if let Transaction::Btc(btc_tx) = tx {
                    if btc_tx.is_same_btc_tx(&upd_tx) {
                        *btc_tx = upd_tx.clone();
                        return
                    }
                }
            }
            // We only reach here if it's a previously unseen transaction. Otherwise we return in the for loop
            self.balances.entry(Currency::BTC).and_modify(|v| *v += upd_tx.amount).or_insert(upd_tx.amount);
            self.deposits.push(Transaction::Btc(upd_tx));
        }
    }

    pub fn cancel_btc_tx(&mut self, canceled_tx: BtcTxCancel) {
        let mut remove_i = None;
        for (i, tx) in self.deposits.iter().enumerate() {
            match tx {
                Transaction::Btc(btc_tx) if btc_tx.is_same_btc_tx(&canceled_tx) => {
                    remove_i = Some(i);
                    break;
                }
                _ => (),
            }
        }
        if let Some(i) = remove_i {
            self.deposits.remove(i);
            self.balances.entry(Currency::BTC).and_modify(|v| *v -= canceled_tx.amount as i64).or_insert(-(canceled_tx.amount as i64));
        }
    }
}
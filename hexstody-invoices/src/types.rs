use chrono::{DateTime, Utc};
use hexstody_api::domain::Currency;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum PaymentMethod {
    /// Onchain payment
    Onchain,
    /// L2 payment: Lightning, Polygon etc 
    L2
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum InvoiceStatus {
    Created,
    TimedOut,
    /// Reason for cancelation
    Canceled(String),
    /// Proof of payment
    Paid(String),
    /// In case payer have overpaid for some reason. + Proof of payment
    Overpaid(String),
    /// In case payer have underpaid. + Proof of payment
    Underpaid(String)
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
/// Same thing as InvoiceStatus, but carries no additional info, so it's easier to check for equality
pub enum InvoiceStatusTag {
    Created,
    TimedOut,
    Canceled,
    Paid,
    Overpaid,
    Underpaid
}

impl InvoiceStatus {
    pub fn is_rejected(&self) -> bool {
        match self {
            Self::TimedOut => true,
            Self::Canceled(_) => true,
            _ => false
        }
    }

    pub fn is_pending(&self) -> bool{
        match self {
            Self::Created => true,
            Self::Underpaid(_) => true,
            _ => false
        }
    }

    pub fn is_completed(&self) -> bool{
        match self {
            Self::Paid(_) => true,
            Self::Overpaid(_) => true,
            _ => false
        }
    }

    pub fn to_tag(&self) -> InvoiceStatusTag{
        match self {
            InvoiceStatus::Created => InvoiceStatusTag::Created,
            InvoiceStatus::TimedOut => InvoiceStatusTag::TimedOut,
            InvoiceStatus::Canceled(_) => InvoiceStatusTag::Canceled,
            InvoiceStatus::Paid(_) => InvoiceStatusTag::Paid,
            InvoiceStatus::Overpaid(_) => InvoiceStatusTag::Overpaid,
            InvoiceStatus::Underpaid(_) => InvoiceStatusTag::Underpaid,
        }
    }
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq
)]
pub struct ContactInfo{
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tg_name: Option<String>
}

impl Default for ContactInfo {
    fn default() -> ContactInfo {
        ContactInfo { 
            email: None,
            phone: None,
            tg_name: None 
        }
    }
}

impl ContactInfo {
    pub fn is_none(&self) -> bool {
        self.email.is_none() && self.phone.is_none() && self.tg_name.is_none()
    }
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq
)]
pub struct Invoice {
    /// Invoice id in hexstody state
    pub id: Uuid,
    /// User who created the invoice
    pub user: String,
    /// Invoice currency
    pub currency: Currency,
    /// Payment method: Onchain or Level2
    pub payment_method: PaymentMethod,
    /// Address to pay to. Allocated by hexstody
    pub address: String,
    /// Amount to pay
    pub amount: u64,
    /// Created. Determined by hexstody
    pub created: DateTime<Utc>,
    /// Due date: sent by requester
    pub due: DateTime<Utc>,
    /// External id used by merchant to identify an invoice
    pub order_id: String,
    /// Optional info to contact payer in case of complications
    pub contact_info: Option<ContactInfo>,
    /// Invoice description
    pub description: String,
    /// Callback url to signal that the invoice is paid for
    pub callback: Option<String>,
    /// Invoice status
    pub status: InvoiceStatus
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq
)]
pub struct CreateInvoiceReq {
    pub currency: Currency,
    pub payment_method: PaymentMethod,
    pub amount: u64,
    pub due: DateTime<Utc>,
    pub order_id: String,
    pub contact_info: Option<ContactInfo>,
    pub description: String,
    pub callback: Option<String>
}
use async_trait::async_trait;
use hexstody_api::domain::Currency;
use uuid::Uuid;

use crate::{types::{Invoice, InvoiceStatus, InvoiceStatusTag}, error};

#[async_trait]
pub trait InvoiceStorage {
    async fn get_invoice(&self, id: &Uuid) -> Option<Invoice>;
    async fn get_user_invoice(&self, user: &String, id: &Uuid) -> Option<Invoice>;
    async fn store_invoice(&mut self, invoice: Invoice) -> error::Result<()>;
    async fn get_user_invoices(&self, user: &String) -> Vec<Invoice>;
    async fn allocate_invoice_address(&mut self, user: &String, currency: &Currency) -> error::Result<String>;
    async fn get_invoices_by_status(&self, status: InvoiceStatusTag) -> Vec<Invoice>;
    async fn set_invoice_status(&mut self, user: &String, id: Uuid, status: InvoiceStatus) -> error::Result<()>;
    async fn set_invoice_status_batch(&mut self, vals: Vec<(String, Uuid, InvoiceStatus)>) -> error::Result<()>;
}
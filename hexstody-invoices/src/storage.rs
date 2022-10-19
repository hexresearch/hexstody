use async_trait::async_trait;
use hexstody_api::domain::Currency;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::{types::{Invoice, InvoiceStatus, InvoiceStatusTag}, error};

#[async_trait]
pub trait InvoiceStorage {
    type Update;
    fn get_invoice(&self, id: &Uuid) -> Option<Invoice>;
    fn get_user_invoice(&self, user: &String, id: &Uuid) -> Option<Invoice>;
    fn get_user_invoices(&self, user: &String) -> Vec<Invoice>;
    fn get_invoices_by_status(&self, status: InvoiceStatusTag) -> Vec<Invoice>;
    async fn store_invoice(&mut self, sender: &Sender<Self::Update>, invoice: Invoice) -> error::Result<()>;
    async fn allocate_invoice_address(&mut self, sender: &Sender<Self::Update>, user: &String, currency: &Currency) -> error::Result<String>;
    async fn set_invoice_status(&mut self, sender: &Sender<Self::Update>, user: &String, id: Uuid, status: InvoiceStatus) -> error::Result<()>;
    async fn set_invoice_status_batch(&mut self, sender: &Sender<Self::Update>, vals: Vec<(String, Uuid, InvoiceStatus)>) -> error::Result<()>;
}
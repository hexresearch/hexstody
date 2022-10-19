use std::sync::Arc;

use log::info;
use tokio::sync::{Mutex, mpsc::Sender};

use crate::{storage::InvoiceStorage, types::{InvoiceStatusTag, InvoiceStatus}};

/// Delay between refreshes in seconds
static REFRESH_PERIOD: u64 = 60;

pub async fn invoice_cleanup_worker<S>(
    state: Arc<Mutex<S>>,
    updater: Sender<S::Update>)
where S: InvoiceStorage + Send
{
    info!("Started invoice cleanup worker with period {}s", REFRESH_PERIOD);
    let mut period = tokio::time::interval(tokio::time::Duration::from_secs(REFRESH_PERIOD));
    loop {
        period.tick().await;
        let now = chrono::offset::Utc::now();
        let mut state = state.lock().await;
        let invs = state.get_invoices_by_status(InvoiceStatusTag::Created);
        
        let req = invs.into_iter().filter_map(|inv| if inv.due <= now {
            Some((inv.user, inv.id, InvoiceStatus::TimedOut))
        } else {
            None
        }).collect();
        let _ = state.set_invoice_status_batch(&updater, req);
    }
}
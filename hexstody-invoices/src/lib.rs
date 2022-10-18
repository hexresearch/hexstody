pub mod types;
pub mod routes;
pub mod storage;
pub mod error;
pub mod worker;

#[cfg(test)]
mod tests{
    use std::collections::HashMap;
    use async_trait::async_trait;
    use hexstody_api::domain::Currency;
    use uuid::Uuid;

    use crate::error;
    use crate::types::*;
    use crate::storage::*;

    struct TestStorage{
        pub invoices: HashMap<String, HashMap<Uuid, Invoice>>
    }
    
    #[async_trait]
    impl InvoiceStorage for TestStorage{
        async fn get_invoice(&self, id: &Uuid) -> Option<Invoice> {
            for invs in self.invoices.values(){
                let r = invs.get(id);
                if r.is_some() {
                    return r.cloned()
                }
            }
            None
        }

        async fn get_user_invoice(&self, user: &String, id: &Uuid) -> Option<Invoice> {
            self.invoices.get(user).map(|invs| invs.get(id)).flatten().cloned()
        }

        async fn store_invoice(&mut self, invoice: Invoice) -> error::Result<()> {
            let user = invoice.user.clone();
            let id = invoice.id.clone();
            self.invoices
                .entry(user)
                .and_modify(|invs| {
                    invs.insert(id, invoice.clone());
                })
                .or_insert(
                    HashMap::from([(id, invoice)])
                );
            Ok(())
        }

        async fn get_user_invoices(&self, user: &String) -> Vec<Invoice> {
            self.invoices.get(user)
                .map(|invs| 
                    invs.values().cloned().collect()
                ).unwrap_or(vec![])
        }

        async fn allocate_invoice_address(&mut self, _: &String, _: &Currency) -> error::Result<String> {
            Ok("testaddress".to_string())
        }

        async fn get_invoices_by_status(&self, status: InvoiceStatusTag) -> Vec<Invoice> {
            self.invoices.values().flat_map(|ivs| ivs.values().filter(|inv| inv.status.to_tag() == status)).cloned().collect()
        }

        async fn set_invoice_status(&mut self, user: &String, id: Uuid, status: InvoiceStatus) -> error::Result<()> {
            self.invoices.get_mut(user)
                .ok_or(crate::error::Error::GenericError("User not found".to_string()))?
                .get_mut(&id)
                .ok_or(crate::error::Error::GenericError("Invoice not found".to_string()))?
                .status = status;
            Ok(())
        }

        async fn set_invoice_status_batch(&mut self, vals: Vec<(String, Uuid, InvoiceStatus)>) -> error::Result<()> {
            for (u, i, s) in vals{
                if let Some(invs) = self.invoices.get_mut(&u) {
                    if let Some(inv) = invs.get_mut(&i) {
                        inv.status = s
                    }
                }
            }
            Ok(())
        }
    }

    fn dummy_invoice() -> Invoice {
        Invoice { 
            id: Uuid::new_v4(),
            user: "testuser".to_string(),
            currency: hexstody_api::domain::Currency::BTC,
            payment_method: PaymentMethod::Onchain,
            address: "testaddress".to_string(),
            amount: 0,
            created: chrono::offset::Utc::now(),
            due: chrono::offset::Utc::now(),
            order_id: "testorderid".to_string(),
            contact_info: Some(ContactInfo::default()),
            description: "testdescription".to_string(),
            callback: Some("testurl".to_string()),
            status: InvoiceStatus::Created 
        }
    }

    #[tokio::test]
    async fn insert_and_retrieve() {
        let mut state = TestStorage{ invoices: HashMap::new()}; 
        let invoice = dummy_invoice();
        let invoice2 = invoice.clone();
        state.store_invoice(invoice).await.expect("Failed to store invoice");
        let resp = state.get_invoice(&invoice2.id).await;
        assert_eq!(resp,Some(invoice2));
    }

    #[tokio::test]
    async fn test_status() {
        let mut state = TestStorage{ invoices: HashMap::new()}; 
        let invoice = dummy_invoice();
        let invoice_orig = invoice.clone();
        state.store_invoice(invoice).await.expect("Failed to store invoice");
        let resp = state.get_invoices_by_status(InvoiceStatusTag::Created).await;
        assert_eq!(resp,vec![invoice_orig.clone()]);

        state.set_invoice_status(&invoice_orig.user, invoice_orig.id.clone(), InvoiceStatus::Paid("()".to_string()))
            .await
            .expect("Failed to set status");

        let resp = state.get_invoices_by_status(InvoiceStatusTag::Created).await;
        assert_eq!(resp,vec![]);
        
        let resp = state.get_invoices_by_status(InvoiceStatusTag::Paid).await;
        assert_eq!(resp.len(), 1);
    }
}
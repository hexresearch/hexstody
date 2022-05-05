use thiserror::Error;
use log::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("dummy")]
    Dummy(),
}

pub async fn node_worker() -> Result<(), Error> {
    loop {
        info!("Node worker thinking...");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        return Err(Error::Dummy());
    }
}
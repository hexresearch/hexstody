use hexstody_api::domain::*;
use hexstody_api::types::*;
use log::*;
use p256::SecretKey;
use p256::ecdsa::SigningKey;
use p256::ecdsa::signature::Signer;
use p256::pkcs8::EncodePublicKey;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Requesting server error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON encoding/decoding error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Alias for a `Result` with the error type `self::Error`.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct HexstodyClient {
    pub client: reqwest::Client,
    pub server: String,
    pub operator: String
}

impl HexstodyClient {
    pub fn new(url: &str, op_url: &str) -> reqwest::Result<Self> {
        Ok(HexstodyClient {
            client: reqwest::ClientBuilder::new().cookie_store(true).build()?,
            server: url.to_owned(),
            operator: op_url.to_owned()
        })
    }

    pub async fn ping(&self) -> Result<()> {
        let path = "/ping";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.get(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(())
    }

    pub async fn signup_email(&self, data: SignupEmail) -> Result<()> {
        let path = "/signup/email";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).json(&data).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(())
    }

    pub async fn signin_email(&self, data: SigninEmail) -> Result<()> {
        let path = "/signin/email";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).json(&data).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(())
    }

    pub async fn logout(&self) -> Result<()> {
        let path = "/logout";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.get(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(())
    }

    pub async fn get_balance(&self) -> Result<Balance> {
        let path = "/balance";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.get(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(serde_json::from_str(&response)?)
    }

    pub async fn get_deposit_address(&self, currency: Currency) -> Result<DepositInfo> {
        let path = "/deposit/address";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).json(&currency).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(serde_json::from_str(&response)?)
    }

    pub async fn eth_ticker(&self, currency: Currency) -> Result<TickerETH> {
        let path = "/ethticker";
        let endpoint = format!("{}{}", self.server, path);
        let request = self.client.post(endpoint).json(&currency).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(serde_json::from_str(&response)?)
    }

    // This function is used for test purposes only
    pub async fn test_only_remove_eth_user(&self, user: &str) -> Result<()> {
        let path = "/removeuser";
        let endpoint = format!("{}{}/{}", self.server, path, user);
        let request = self.client.get(endpoint).build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("Response {path}: {}", response);
        Ok(())
    }

    /// This function is used for test purposes only
    pub async fn gen_invite(&self, secret_key: SecretKey, invite_req: InviteRequest) -> Result<InviteResp> {
        let path = "/invite/generate";
        let endpoint = format!("{}{}", self.operator, path);
        let nonce = 0;
        let message = rocket::serde::json::to_string(&invite_req).unwrap();
        let message_to_sign = [endpoint.clone(), message.clone(), nonce.to_string()].join(":");
        let pk_str = base64::encode(secret_key.clone().public_key().to_public_key_der().unwrap().to_vec());
        let signature = SigningKey::from(secret_key.clone()).sign(message_to_sign.as_bytes());
        let sig_str = base64::encode(signature.to_der().as_bytes());
        let signature_data = format!("{}:{}:{}", sig_str, nonce, pk_str);
        let request = self.client
            .post(endpoint)
            .json(&invite_req)
            .header("Content-Type", "application/json")
            .header("Signature-Data", signature_data)
            .build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(serde_json::from_str(&response)?)
    }
}

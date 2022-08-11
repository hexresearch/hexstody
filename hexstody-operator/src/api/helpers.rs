use hexstody_api::types::SignatureData;
use hexstody_api::error;
use hexstody_sig::SignatureVerificationData;
use p256::PublicKey;
use rocket::serde::json;
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct Config {
    pub domain: String,
    pub operator_public_keys: Vec<PublicKey>,
}

/// Guard operator handle from non-authorized user
pub fn guard_op_signature<T: Serialize>(
    config: &Config, 
    uri: String, 
    signature_data: SignatureData,
    body: &T
) -> error::Result<()>{
    let url = [config.domain.clone(), uri].join("");
    let message = json::to_string(body).unwrap();
    let signature_verification_data = SignatureVerificationData {
        url,
        signature: signature_data.signature,
        nonce: signature_data.nonce,
        message: Some(message),
        public_key: signature_data.public_key,
    };
    signature_verification_data
        .verify(config.operator_public_keys.clone())
        .map_err(|e| error::Error::SignatureError(format!("{:?}", e)).into())
}

/// Guard operator handle from non-authorized user. Special case for when request has no body attached
pub fn guard_op_signature_nomsg(
    config: &Config, 
    uri: String, 
    signature_data: SignatureData,
) -> error::Result<()>{
    let url = [config.domain.clone(), uri].join("");
    let signature_verification_data = SignatureVerificationData {
        url,
        signature: signature_data.signature,
        nonce: signature_data.nonce,
        message: None,
        public_key: signature_data.public_key,
    };
    signature_verification_data
        .verify(config.operator_public_keys.clone())
        .map_err(|e| error::Error::SignatureError(format!("{:?}", e)).into())
}
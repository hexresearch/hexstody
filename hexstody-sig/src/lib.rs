use p256::{
    ecdsa::{signature::Verifier, Signature, VerifyingKey},
    PublicKey,
};
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidPublicKey,
    InvalidSignature,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidPublicKey => write!(f, "Invalid public key"),
            Error::InvalidSignature => write!(f, "Invalid signature"),
        }
    }
}

impl StdError for Error {}

#[derive(Debug, PartialEq)]
pub struct SignatureVerificationData {
    pub url: String,
    pub signature: Signature,
    pub nonce: u64,
    pub public_key: PublicKey,
    pub message: Option<String>,
    pub operator_public_keys: Vec<PublicKey>,
}

impl SignatureVerificationData {
    pub fn verify(&self) -> Result<(), Error> {
        if !self.operator_public_keys.contains(&self.public_key) {
            return Err(Error::InvalidPublicKey);
        };
        let message_items = match self.message.clone() {
            None => vec![self.url.clone(), self.nonce.to_string()],
            Some(msg) => vec![self.url.clone(), msg, self.nonce.to_string()],
        };
        let message = message_items.join(":");
        VerifyingKey::from(self.public_key)
            .verify(message.as_bytes(), &self.signature)
            .map_err(|_| Error::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use p256::{
        ecdsa::{signature::Signer, SigningKey},
        SecretKey,
    };
    use rand_core::OsRng;

    use crate::{Error, SignatureVerificationData};

    #[test]
    fn test_signature() {
        let url = "test_url".to_owned();
        let nonce = 0;
        let message = "test_message".to_owned();
        let message_to_sign = [url.clone(), message.clone(), nonce.to_string()].join(":");
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        let signature = SigningKey::from(secret_key).sign(message_to_sign.as_bytes());
        let signature_verification_data = SignatureVerificationData {
            url,
            signature,
            nonce,
            public_key,
            message: Some(message),
            operator_public_keys: vec![public_key],
        };
        assert_eq!(signature_verification_data.verify(), Ok(()));
    }

    #[test]
    fn test_signature_empty_message() {
        let url = "test_url".to_owned();
        let nonce = 0;
        let message_to_sign = [url.clone(), nonce.to_string()].join(":");
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        let signature = SigningKey::from(secret_key).sign(message_to_sign.as_bytes());
        let signature_verification_data = SignatureVerificationData {
            url,
            signature,
            nonce,
            public_key,
            // The message is empty
            message: None,
            operator_public_keys: vec![public_key],
        };
        assert_eq!(signature_verification_data.verify(), Ok(()));
    }

    #[test]
    fn test_signature_invalid_key() {
        let url = "test_url".to_owned();
        let nonce = 0;
        let message = "test_message".to_owned();
        let message_to_sign = [url.clone(), message.clone(), nonce.to_string()].join(":");
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        let signature = SigningKey::from(secret_key).sign(message_to_sign.as_bytes());
        let signature_verification_data = SignatureVerificationData {
            url,
            signature,
            nonce,
            public_key,
            message: None,
            // No trusted keys
            operator_public_keys: vec![],
        };
        assert_eq!(
            signature_verification_data.verify(),
            Err(Error::InvalidPublicKey)
        );
    }

    #[test]
    fn test_signature_wrong_msg() {
        let url = "test_url".to_owned();
        let nonce = 0;
        let message = "test_message".to_owned();
        let message_to_sign = [url.clone(), message.clone(), nonce.to_string()].join(":");
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        let signature = SigningKey::from(secret_key).sign(message_to_sign.as_bytes());
        let signature_verification_data = SignatureVerificationData {
            url,
            signature,
            // Here we change the nonce so that the signature becomes invalid
            nonce: nonce + 1,
            public_key,
            message: None,
            operator_public_keys: vec![public_key],
        };
        assert_eq!(
            signature_verification_data.verify(),
            Err(Error::InvalidSignature)
        );
    }
}

use p256::{
    ecdsa::{signature::Verifier, Signature, VerifyingKey},
    PublicKey,
};

#[derive(Debug, PartialEq)]
pub enum SignatureError {
    InvalidPublicKey,
    InvalidSignature,
    InvalidDomain,
}

#[derive(Debug, PartialEq)]
pub struct SignatureVerificationData {
    pub url: String,
    pub signature: Signature,
    pub nonce: u64,
    pub public_key: PublicKey,
    pub message: Option<String>,
}

impl SignatureVerificationData {
    pub fn verify(&self, operator_public_keys: Vec<PublicKey>) -> Result<(), SignatureError> {
        if !operator_public_keys.contains(&self.public_key) {
            return Err(SignatureError::InvalidPublicKey);
        };
        let message_items = match self.message.clone() {
            None => vec![self.url.clone(), self.nonce.to_string()],
            Some(msg) => vec![self.url.clone(), msg, self.nonce.to_string()],
        };
        let message = message_items.join(":");
        VerifyingKey::from(self.public_key)
            .verify(message.as_bytes(), &self.signature)
            .map_err(|_| SignatureError::InvalidSignature)
    }
}

pub fn verify_signature(
    operator_public_keys: Option<Vec<PublicKey>>,
    public_key: &PublicKey,
    nonce: &u64, 
    message: String,
    signature: &Signature
) -> Result<(), SignatureError>{
    if operator_public_keys.map(|pks| !pks.contains(&public_key)).unwrap_or(false) {
        return Err(SignatureError::InvalidPublicKey);
    };
    let message = [message, nonce.to_string()].join(":");
    VerifyingKey::from(public_key)
        .verify(message.as_bytes(), &signature)
        .map_err(|_| SignatureError::InvalidSignature)
}

#[cfg(test)]
mod tests {
    use p256::{
        ecdsa::{signature::Signer, SigningKey},
        SecretKey,
    };
    use rand_core::OsRng;

    use crate::{SignatureVerificationData, SignatureError, verify_signature};

    #[test]
    fn test_signature_svd() {
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
        };
        assert_eq!(signature_verification_data.verify(vec![public_key]), Ok(()));
    }

    #[test]
    fn test_signature_empty_message_svd() {
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
        };
        assert_eq!(signature_verification_data.verify(vec![public_key]), Ok(()));
    }

    #[test]
    fn test_signature_invalid_key_svd() {
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
        };
        // No trusted keys
        assert_eq!(signature_verification_data.verify(vec![]), Err(SignatureError::InvalidPublicKey));
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
        };
        assert_eq!(signature_verification_data.verify(vec![public_key]), Err(SignatureError::InvalidSignature));
    }

    #[test]
    fn test_signature() {
        let nonce = 0;
        let message = "test_message".to_owned();
        let message_to_sign = [message.clone(), nonce.to_string()].join(":");
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        let signature = SigningKey::from(secret_key).sign(message_to_sign.as_bytes());
        assert_eq!(verify_signature(Some(vec![public_key]), &public_key, &nonce, message, &signature), Ok(()));
    }

    #[test]
    fn test_signature_no_keys_required() {
        let nonce = 0;
        let message = "test_message".to_owned();
        let message_to_sign = [message.clone(), nonce.to_string()].join(":");
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        let signature = SigningKey::from(secret_key).sign(message_to_sign.as_bytes());
        assert_eq!(verify_signature(None, &public_key, &nonce, message, &signature), Ok(()));
    }

    #[test]
    fn test_signature_no_trusted_keys() {
        let nonce = 0;
        let message = "test_message".to_owned();
        let message_to_sign = [message.clone(), nonce.to_string()].join(":");
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        let signature = SigningKey::from(secret_key).sign(message_to_sign.as_bytes());
        assert_eq!(verify_signature(Some(vec![]), &public_key, &nonce, message, &signature), Err(SignatureError::InvalidPublicKey));
    }

    #[test]
    fn test_signature_invalid_sig(){
        let nonce = 0;
        let notnonce = 1;
        let message = "test_message".to_owned();
        let message_to_sign = [message.clone(), nonce.to_string()].join(":");
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        let signature = SigningKey::from(secret_key).sign(message_to_sign.as_bytes());
        assert_eq!(verify_signature(None, &public_key, &notnonce, message, &signature), Err(SignatureError::InvalidSignature));
    }
}

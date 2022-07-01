use p256::{
    ecdsa::{signature::Verifier, Signature, VerifyingKey},
    PublicKey,
};

use hexstody_api::types::WithdrawalSignature;

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
    pub operator_public_keys: Vec<PublicKey>,
}

impl SignatureVerificationData {
    pub fn verify(&self) -> Result<(), SignatureError> {
        if !self.operator_public_keys.contains(&self.public_key) {
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


pub fn verify_withdrawal_signature(
    operator_public_keys: Option<Vec<PublicKey>>,
    withdrawal_signature: &WithdrawalSignature,
    domain: String,
    message: String
) -> Result<(), SignatureError>{
    let WithdrawalSignature { signature, public_key, nonce, verdict } = withdrawal_signature;
    let msg = [message, domain, verdict.to_json()].join(":");
    verify_signature(operator_public_keys, public_key, nonce, msg, signature)
}

#[cfg(test)]
mod tests {
    use hexstody_api::types::{WithdrawalSignature, WithdrawalRequestDecisionType};
    use p256::{
        ecdsa::{signature::Signer, SigningKey},
        SecretKey,
    };
    use rand_core::OsRng;

    use crate::{SignatureVerificationData, SignatureError, verify_signature, verify_withdrawal_signature};

    fn mk_withdrawal_signature(secret_key: &SecretKey) -> WithdrawalSignature{
        let nonce = 0;
        let message = "test_message".to_owned();
        let domain = "hexstody-hot".to_owned();
        let verdict = WithdrawalRequestDecisionType::Confirm;
        let message_to_sign = [message.clone(), domain, verdict.to_json(), nonce.to_string()].join(":");
        let public_key = secret_key.public_key().clone();
        let signature = SigningKey::from(secret_key).sign(message_to_sign.as_bytes());
        WithdrawalSignature { signature, public_key, nonce, verdict }
    }

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
            operator_public_keys: vec![public_key],
        };
        assert_eq!(signature_verification_data.verify(), Ok(()));
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
            operator_public_keys: vec![public_key],
        };
        assert_eq!(signature_verification_data.verify(), Ok(()));
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
            // No trusted keys
            operator_public_keys: vec![],
        };
        assert_eq!(signature_verification_data.verify(), Err(SignatureError::InvalidPublicKey));
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
        assert_eq!(signature_verification_data.verify(), Err(SignatureError::InvalidSignature));
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

//

    #[test]
    fn test_signature_ws() {
        let secret_key = SecretKey::random(&mut OsRng);
        let message = "test_message".to_owned();
        let ws = mk_withdrawal_signature(&secret_key);
        let domain = "hexstody-hot".to_owned();
        let public_key = secret_key.public_key();
        assert_eq!(verify_withdrawal_signature(Some(vec![public_key]), &ws, domain, message), Ok(()));
    }

    #[test]
    fn test_signature_no_keys_required_ws() {
        let secret_key = SecretKey::random(&mut OsRng);
        let message = "test_message".to_owned();
        let ws = mk_withdrawal_signature(&secret_key);
        let domain = "hexstody-hot".to_owned();
        assert_eq!(verify_withdrawal_signature(None, &ws, domain, message), Ok(()));
    }

    #[test]
    fn test_signature_no_trusted_keys_ws() {
        let secret_key = SecretKey::random(&mut OsRng);
        let message = "test_message".to_owned();
        let ws = mk_withdrawal_signature(&secret_key);
        let domain = "hexstody-hot".to_owned();
        assert_eq!(verify_withdrawal_signature(Some(vec![]), &ws, domain, message), Err(SignatureError::InvalidPublicKey));
    }

    #[test]
    fn test_signature_invalid_sig_ws(){
        let secret_key = SecretKey::random(&mut OsRng);
        let message = "wrong_message".to_owned();
        let ws = mk_withdrawal_signature(&secret_key);
        let domain = "hexstody-hot".to_owned();
        assert_eq!(verify_withdrawal_signature(None, &ws, domain, message), Err(SignatureError::InvalidSignature));
    }
}

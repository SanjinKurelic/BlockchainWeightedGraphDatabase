use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;

pub struct Wallet {
    signing_key: SigningKey,
}

impl Default for Wallet {
    fn default() -> Self {
        Wallet {
            signing_key: SigningKey::generate(&mut OsRng),
        }
    }
}

impl Wallet {
    pub(crate) fn get_public_key(&self) -> String {
        hex::encode(self.signing_key.verifying_key().to_bytes())
    }

    pub fn sign(&mut self, hash: &String) -> String {
        hex::encode(self.signing_key.sign(hash.as_bytes()).to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    use std::str::FromStr;

    #[test]
    fn should_sign_message() {
        // Given
        let mut wallet = Wallet::default();
        let message = "test".to_string();

        // When
        let signature = wallet.sign(&message);

        // Then
        assert_eq!(wallet.get_public_key(), hex::encode(wallet.signing_key.verifying_key().to_bytes()));
        assert!(wallet
            .signing_key
            .verify(message.as_bytes(), &Signature::from_str(signature.as_str()).unwrap())
            .is_ok());

        let public_key = VerifyingKey::from_bytes(hex::decode(wallet.get_public_key()).unwrap().as_slice().try_into().unwrap()).unwrap();
        assert!(public_key
            .verify(message.as_bytes(), &Signature::from_str(signature.as_str()).unwrap())
            .is_ok());
    }
}

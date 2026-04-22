use crate::error::AppError;
use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use ed25519_dalek::pkcs8::spki::der::zeroize::{Zeroize, Zeroizing};
use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};
use getrandom::fill;

#[derive(Debug)]
pub(crate) struct KeyPair {
    signing_key: SigningKey,
}

impl KeyPair {
    pub(crate) fn generate() -> Result<Self, AppError> {
        let mut seed = [0u8; 32];
        fill(&mut seed).map_err(|_| AppError::internal().with_details("Entropy source failure"))?;

        let signing_key = SigningKey::from_bytes(&seed);
        seed.zeroize();

        Ok(Self { signing_key })
    }

    pub(crate) fn private_key_pem(&self) -> Result<Zeroizing<String>, AppError> {
        self.signing_key.to_pkcs8_pem(LineEnding::LF).map_err(|e| {
            AppError::internal().with_details(format!("Private key PEM encoding failed: {e}"))
        })
    }

    pub(crate) fn public_key_pem(&self) -> Result<String, AppError> {
        self.signing_key.verifying_key().to_public_key_pem(LineEnding::LF).map_err(|e| {
            AppError::internal().with_details(format!("Public key PEM encoding failed: {e}"))
        })
    }
}

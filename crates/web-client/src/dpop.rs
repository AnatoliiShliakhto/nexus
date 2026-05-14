//! DPoP (Demonstrating Proof-of-Possession) proof generation.
//!
//! This module provides the [`DpopSigner`] which handles ephemeral Ed25519 key generation
//! and JWT-based proof signing as per RFC 9449.

use crate::error::WebClientError;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::SigningKey;
use jsonwebtoken::crypto::CryptoProvider;
use jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER;
use jsonwebtoken::jwk::{
    AlgorithmParameters, CommonParameters, EllipticCurve, Jwk, KeyAlgorithm,
    OctetKeyPairParameters, OctetKeyPairType,
};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// An internal signer that manages ephemeral keys and JWT headers for DPoP proofs.
#[derive(Debug)]
pub(crate) struct DpopSigner {
    key: EncodingKey,
    header: Header,
}

/// DPoP JWT Claims as defined in RFC 9449 Section 4.2.
#[derive(Debug, Serialize)]
struct DpopClaims<'a> {
    /// Unique identifier for the proof to prevent replay attacks.
    jti: String,
    /// The HTTP method of the request.
    htm: &'a str,
    /// The HTTP URI of the request (without query and fragment).
    htu: &'a str,
    /// Issued-at timestamp.
    iat: i64,
    /// Hash of the access token, if the proof is bound to one.
    #[serde(skip_serializing_if = "Option::is_none")]
    ath: Option<String>,
}

impl DpopSigner {
    /// Initializes a new signer by generating an ephemeral Ed25519 key pair.
    ///
    /// This also sets up the JWK (JSON Web Key) in the header for public key embedding.
    ///
    /// # Errors
    ///
    /// Returns [`WebClientError`] if random number generation or key encoding fails.
    pub(crate) fn init() -> Result<Self, WebClientError> {
        // Ensure the default crypto provider is installed for EdDSA support.
        let _ = CryptoProvider::install_default(&DEFAULT_PROVIDER);

        let signing_key = generate_signing_key()?;

        // Construct the JWK containing the public key.
        let jwk = Jwk {
            common: CommonParameters {
                key_algorithm: Some(KeyAlgorithm::EdDSA),
                ..Default::default()
            },
            algorithm: AlgorithmParameters::OctetKeyPair(OctetKeyPairParameters {
                key_type: OctetKeyPairType::OctetKeyPair,
                curve: EllipticCurve::Ed25519,
                x: URL_SAFE_NO_PAD.encode(signing_key.verifying_key().as_bytes()),
            }),
        };

        let mut header = Header::new(Algorithm::EdDSA);
        header.typ = Some("dpop+jwt".to_owned());
        header.jwk = Some(jwk);

        // Convert the raw seed to PKCS#8 v2 DER format for `jsonwebtoken`.
        let pkcs8_key = to_pkcs8_v2(signing_key.as_bytes());
        let key = EncodingKey::from_ed_der(&pkcs8_key);

        Ok(Self { key, header })
    }

    /// Generates a signed DPoP proof string for a specific request.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method (e.g., "GET", "POST").
    /// * `url` - The destination URL.
    /// * `access_token` - Optional access token to bind via the `ath` claim.
    ///
    /// # Errors
    ///
    /// Returns [`WebClientError`] if JWT encoding fails.
    pub(crate) fn prove(
        &self,
        method: &str,
        url: &str,
        access_token: Option<&str>,
    ) -> Result<String, WebClientError> {
        let ath = access_token.map(|token| {
            let mut hasher = Sha256::new();
            hasher.update(token.as_bytes());
            URL_SAFE_NO_PAD.encode(hasher.finalize())
        });

        let claims = DpopClaims {
            jti: Uuid::new_v4().to_string(),
            htm: method,
            htu: url,
            iat: chrono::Utc::now().timestamp(),
            ath,
        };

        encode(&self.header, &claims, &self.key).map_err(WebClientError::from)
    }
}
// --- Helpers ---

/// Generates a cryptographically secure 32-byte Ed25519 signing key.
fn generate_signing_key() -> Result<SigningKey, WebClientError> {
    let mut bytes = [0u8; 32];

    getrandom::fill(&mut bytes)
        .map_err(|e| WebClientError::internal().with_details(e.to_string()))?;

    Ok(SigningKey::from_bytes(&bytes))
}

/// Wraps a 32-byte Ed25519 seed into a PKCS#8 v2 Unencrypted Private Key Info structure.
///
/// This is a manual DER encoding to avoid heavy dependencies like `der` or `pkcs8` crates
/// in the WASM-focused client.
fn to_pkcs8_v2(seed: &[u8; 32]) -> Vec<u8> {
    let mut res = Vec::with_capacity(48);
    res.extend_from_slice(&[
        0x30, 0x2e, // Sequence, length 46
        0x02, 0x01, 0x00, // Version 0
        0x30, 0x05, // Algorithm Identifier Sequence
        0x06, 0x03, 0x2b, 0x65, 0x70, // OID 1.3.101.112 (Ed25519)
        0x04, 0x22, // Octet String, length 34
        0x04, 0x20, // Octet String, length 32
    ]);
    res.extend_from_slice(seed);
    res
}

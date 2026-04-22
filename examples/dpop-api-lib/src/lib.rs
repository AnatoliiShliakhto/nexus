//! # `DPoP` API Contract
//!
//! This crate defines the data structures used for communication between
//! the Identity services (Auth, Router) and the `DPoP` validation component.
//!
//! It uses `serde` for serialization, specifically optimized for `postcard`
//! binary transport within the Spin/Wasm runtime.

use serde::{Deserialize, Serialize};

/// Represents the JWT claims of a `DPoP` proof (RFC 9449).
///
/// These claims are used to bind the proof to a specific HTTP request,
/// preventing replay attacks and ensuring the proof is tied to the
/// intended server-side endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProofClaims {
    /// JWT ID (jti): A unique identifier for this `DPoP` proof.
    /// Used by the validator to detect and reject replayed proofs.
    pub jti: String,

    /// HTTP Method (htm): The HTTP method of the request to which this
    /// proof is bound (e.g., "GET", "POST").
    pub htm: String,

    /// HTTP URI (htu): The target HTTP URI (without query parameters)
    /// of the request to which this proof is bound.
    pub htu: String,

    /// Issued At (iat): The time the proof was created, used to verify
    /// the freshness of the `DPoP` proof.
    pub iat: i64,

    /// Access Token Hash (ath): A base64url-encoded SHA-256 hash of
    /// the access token.
    ///
    /// This field is optional and is included when the `DPoP` proof is
    /// bound to a specific access token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ath: Option<String>,

    /// Confirmation (cnf): Contains the `jkt` (JWK Thumbprint) to confirm
    /// the proof is bound to the client's public key.
    pub cnf: Option<Cnf>,
}

/// Represents a request to compute the JWK Thumbprint (JKT) of a client's public key.
/// Typically used by the Authentication service during the login flow.
#[cfg_attr(not(target_arch = "wasm32"), derive(utoipa::ToSchema), schema(as = DpopComputeThumbprint))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeThumbprintRequest {
    /// The raw `DPoP` proof JWT string from the `DPoP` HTTP header.
    pub proof: String,
}

/// Represents the JWK Thumbprint (JKT) for key binding confirmation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Cnf {
    /// JWK Thumbprint (jkt): The base64url-encoded SHA-256 hash of the
    /// JWK used to sign the `DPoP` proof.
    pub jkt: String,
}

/// Represents a request to verify an existing `DPoP` proof against a specific
/// HTTP context and an optional access token.
/// Typically used by the API Gateway or Router for every incoming protected request.
#[cfg_attr(not(target_arch = "wasm32"), derive(utoipa::ToSchema), schema(as = DpopVerify))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRequest {
    /// The raw `DPoP` proof JWT string from the `DPoP` HTTP header.
    pub proof: String,
    /// The HTTP method of the current request (e.g., "GET", "POST").
    pub method: String,
    /// The full HTTP target URI of the current request.
    pub uri: String,
    /// The hash of the access token (`ath`) if the proof is bound to a specific token.
    pub access_token: Option<String>,
}

/// The JWK Thumbprint (JKT) of the client's public key.
///
/// This is a base64url-encoded SHA-256 hash of the JWK, used as a
/// unique identifier for the hardware-bound key.
#[cfg_attr(not(target_arch = "wasm32"), derive(utoipa::ToSchema), schema(as = DpopJkt))]
#[derive(Debug, Serialize, Deserialize)]
pub struct Jkt(pub String);

impl ComputeThumbprintRequest {
    /// Creates a new request for thumbprint computation.
    ///
    /// # Arguments
    /// * `proof` - The raw `DPoP` proof string extracted from the `DPoP` HTTP header.
    pub fn new(proof: impl Into<String>) -> Self {
        Self { proof: proof.into() }
    }
}

impl VerifyRequest {
    /// Creates a new `VerifyRequest` to validate a `DPoP` proof within an HTTP context.
    ///
    /// This request structure is used by downstream services to verify that the
    /// presented `DPoP` proof matches the current HTTP request environment.
    ///
    /// # Arguments
    /// * `proof` - The raw `DPoP` proof string from the client.
    /// * `method` - The HTTP method of the current request (e.g., "POST").
    /// * `uri` - The full request URI to validate against the `htu` claim.
    /// * `access_token` - The optional access token used to validate the `ath` claim.
    pub fn new(
        proof: impl Into<String>,
        method: impl Into<String>,
        uri: impl Into<String>,
        access_token: Option<impl Into<String>>,
    ) -> Self {
        Self {
            proof: proof.into(),
            method: method.into(),
            uri: uri.into(),
            access_token: access_token.map(Into::into),
        }
    }
}

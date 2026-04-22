use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::ops::Deref;

/// Represents the extracted JWT claims of a `DPoP` proof.
#[derive(Debug, Deserialize)]
pub(in crate::features::identity) struct DpopClaims {
    /// JWT ID (jti): Unique identifier to prevent replays.
    pub jti: SmolStr,
    /// HTTP Method (htm): Bound to the request method.
    pub htm: SmolStr,
    /// HTTP URI (htu): Bound to the target URI.
    pub htu: SmolStr,
    /// Issued At (iat): Time the proof was created.
    pub iat: i64,
    /// Access Token Hash (ath): Base64url-encoded SHA-256 hash of the access token.
    pub ath: Option<SmolStr>,
}

/// A wrapper type for the JWK Thumbprint (JKT).
/// Encapsulates the thumbprint string to ensure type safety across the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(in crate::features::identity) struct Jkt(pub(in crate::features::identity) SmolStr);

impl Jkt {
    pub(in crate::features::identity) fn new(val: impl Into<SmolStr>) -> Self {
        Self(val.into())
    }

    pub(in crate::features::identity) fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Jkt {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Jkt> for SmolStr {
    fn from(jkt: Jkt) -> Self {
        jkt.0
    }
}

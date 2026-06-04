use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;
use std::fmt;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Jkt {
    bytes: [u8; 43],
}

impl Jkt {
    pub(crate) fn new(bytes: [u8; 43]) -> Self {
        Self { bytes }
    }

    #[inline]
    pub(crate) fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes) }
    }
}

impl AsRef<str> for Jkt {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Serialize for Jkt {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Jkt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct JktVisitor;

        impl<'de> de::Visitor<'de> for JktVisitor {
            type Value = Jkt;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a 43-byte Base64URL encoded string (JWK Thumbprint)")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let bytes: [u8; 43] = v.as_bytes().try_into().map_err(|_| {
                    E::custom(format!(
                        "invalid JKT length: expected 43 bytes, got {} bytes",
                        v.len()
                    ))
                })?;
                Ok(Jkt { bytes })
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(v)
            }
        }

        deserializer.deserialize_str(JktVisitor)
    }
}

use crate::infra::ip_value::IpValue;
use crate::infra::serde_utils::header_serde;
use bitflags::bitflags;
use fxhash::FxHashMap;
use http::{HeaderValue, Method};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_repr::{Deserialize_repr, Serialize_repr};
use smol_str::SmolStr;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use surrealdb::types::{SurrealValue, Value};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct Actions: u8 {
        const NONE   = 0b0000;
        const CREATE = 0b0001;
        const READ   = 0b0010;
        const UPDATE = 0b0100;
        const DELETE = 0b1000;
        const ALL    = Self::READ.bits() | Self::CREATE.bits() | Self::UPDATE.bits() | Self::DELETE.bits();
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum SessionStatus {
    Pending = 0,
    Active = 1,
    Revoked = 2,
    #[default]
    Expired = 3,
}

impl std::str::FromStr for SessionStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PENDING" => Ok(Self::Pending),
            "ACTIVE" => Ok(Self::Active),
            "EXPIRED" => Ok(Self::Expired),
            "REVOKED" => Ok(Self::Revoked),
            _ => Err(()),
        }
    }
}

impl SessionStatus {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Active => "ACTIVE",
            Self::Revoked => "REVOKED",
            Self::Expired => "EXPIRED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Session {
    pub id: SmolStr,
    pub status: SessionStatus,
    #[serde(with = "header_serde")]
    pub database_token: HeaderValue,
    pub permissions: FxHashMap<String, Actions>,
    pub ip: Option<IpValue>,
    pub jkt: SmolStr,
}

impl Actions {
    pub(crate) fn from_vec(vec: &[impl AsRef<str>]) -> Self {
        vec.iter().fold(Self::NONE, |mut actions, s| {
            match s.as_ref() {
                "CREATE" => actions |= Self::CREATE,
                "READ" => actions |= Self::READ,
                "UPDATE" => actions |= Self::UPDATE,
                "DELETE" => actions |= Self::DELETE,
                _ => {},
            }
            actions
        })
    }
}

impl Session {
    pub(crate) fn check_access(&self, component: impl AsRef<str>, method: &Method) -> bool {
        let required = match method {
            &Method::POST => Actions::CREATE,
            &Method::PUT | &Method::PATCH => Actions::UPDATE,
            _ => Actions::READ,
        };

        self.permissions
            .get(component.as_ref())
            .map(|&allowed| allowed.contains(required))
            .unwrap_or(false)
    }

    pub(crate) fn with_status(&self, status: SessionStatus) -> Arc<Self> {
        Arc::new(Self { status, ..self.clone() })
    }
}

#[derive(SurrealValue)]
#[surreal(tag = "status")]
pub(super) enum AuthResponse {
    #[surreal(rename = "Ok")]
    Ok { data: SessionData },
    #[surreal(rename = "Err")]
    Err { code: String },
}

#[derive(Deserialize, SurrealValue)]
pub(super) struct SessionData {
    pub session_id: String,
    pub account: String,
    pub refresh_token: String,
    pub permissions: HashMap<String, Vec<String>>,
    pub status: String,
}

#[derive(Debug, SurrealValue)]
pub(super) struct SessionChanged {
    pub id: String,
    pub status: String,
}

// --- Helpers ---
impl Serialize for Actions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.bits())
    }
}

impl<'de> Deserialize<'de> for Actions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ActionsVisitor;

        impl<'de> de::Visitor<'de> for ActionsVisitor {
            type Value = Actions;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a numeric bitmask (u8)")
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Actions::from_bits_retain(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value <= u8::MAX as u64 {
                    Ok(Actions::from_bits_retain(value as u8))
                } else {
                    Err(E::custom(format!("bitmask out of range for u8: {value}")))
                }
            }
        }

        deserializer.deserialize_u8(ActionsVisitor)
    }
}

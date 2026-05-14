//! Session management and access control primitives.
//!
//! This module defines [`SessionStatus`] for lifecycle tracking and [`Actions`]
//! for bitmask-based permission handling across system components.

use bitflags::bitflags;
use fxhash::FxHashMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt;

bitflags! {
    /// Bitmask representing granular CRUD permissions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Actions: u8 {
        /// No permissions granted.
        const NONE   = 0b0000;
        /// Permission to create new resources.
        const CREATE = 0b0001;
        /// Permission to view or retrieve resources.
        const READ   = 0b0010;
        /// Permission to modify existing resources.
        const UPDATE = 0b0100;
        /// Permission to remove resources.
        const DELETE = 0b1000;
        /// Full administrative access (CRUD).
        const ALL    = Self::READ.bits() | Self::CREATE.bits() | Self::UPDATE.bits() | Self::DELETE.bits();
    }
}

/// Represents the current lifecycle state of a user session.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum SessionStatus {
    /// Session is created but not yet fully authorized.
    Pending = 0,
    /// Session is active and valid for requests.
    Active = 1,
    /// Session was explicitly terminated by the user or admin.
    Revoked = 2,
    /// Session has naturally timed out.
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
    /// Returns the string representation of the status for logging or protocols.
    #[must_use]
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Active => "ACTIVE",
            Self::Revoked => "REVOKED",
            Self::Expired => "EXPIRED",
        }
    }
}

/// A user session containing status and component-level permissions.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Session {
    /// Current status of the session.
    pub status: SessionStatus,
    /// Permission mapping where the key is the component identifier.
    pub permissions: FxHashMap<String, Actions>,
}

impl Session {
    /// Checks if the session has [`Actions::CREATE`] permission for a component.
    pub fn can_create(&self, component: impl AsRef<str>) -> bool {
        self.permissions.get(component.as_ref()).map_or(false, |a| a.contains(Actions::CREATE))
    }

    /// Checks if the session has [`Actions::READ`] permission for a component.
    pub fn can_read(&self, component: impl AsRef<str>) -> bool {
        self.permissions.get(component.as_ref()).map_or(false, |a| a.contains(Actions::READ))
    }

    /// Checks if the session has [`Actions::UPDATE`] permission for a component.
    pub fn can_update(&self, component: impl AsRef<str>) -> bool {
        self.permissions.get(component.as_ref()).map_or(false, |a| a.contains(Actions::UPDATE))
    }

    /// Checks if the session has [`Actions::DELETE`] permission for a component.
    pub fn can_delete(&self, component: impl AsRef<str>) -> bool {
        self.permissions.get(component.as_ref()).map_or(false, |a| a.contains(Actions::DELETE))
    }
}

// --- Serde Implementation for Actions ---

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

        impl de::Visitor<'_> for ActionsVisitor {
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
                u8::try_from(value).map_or_else(
                    |_| Err(E::custom(format!("bitmask out of range for u8: {value}"))),
                    |v| Ok(Actions::from_bits_retain(v)),
                )
            }
        }

        deserializer.deserialize_u8(ActionsVisitor)
    }
}

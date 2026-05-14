//! # Lockbox
//!
//! `Lockbox` provides a high-level, platform-independent abstraction for secure secret storage.
//! It wraps the `keyring-core` traits to offer a simplified Developer Experience (DX) for
//! managing credentials, API keys, and sensitive JSON data.
//!
//! ## Platform Support
//! - **Desktop (Windows, macOS, Linux):** Automatically selects the native OS credential store (Keychain, Windows Vault, Secret Service).
//! - **WASM/WASI:** Optimized for Spin/Nexus environments, defaulting to SQLite-based storage.
//!
//! ## Core Concepts
//! The primary entry point is the [`Lockbox`] struct. It uses a "service" name to namespace
//! your secrets, preventing collisions with other applications.

use crate::error::{LockboxError, LockboxErrorExt};
use crate::store::{use_named_store, use_native_store};
use keyring_core::get_default_store;
use serde::Serialize;
use std::borrow::Cow;
use std::sync::Arc;

pub mod error;
mod store;

pub use crate::store::NAMED_STORES;

/// A thread-safe, high-level manager for secure secret storage.
///
/// `Lockbox` is designed to be cloned and shared across threads or tasks. It handles
/// the initialization of the underlying storage backend and provides a clean API
/// for CRUD operations on secrets.
///
/// # Example
/// ```rust
/// use lockbox::Lockbox;
///
/// let secrets = Lockbox::new("my-app-name").unwrap();
/// secrets.set("api-key", "super-secret-value").unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Lockbox {
    service: Arc<Cow<'static, str>>,
}

impl Lockbox {
    /// Initializes a new Lockbox instance with the best available native store.
    ///
    /// On non-WASM platforms, it automatically detects the OS and sets up the native
    /// credential store. On WASM, it attempts to initialize an `SQLite` backend.
    ///
    /// # Errors
    /// Returns [`LockboxError::Core`] if the native store fails to initialize or
    /// [`LockboxError::NotSupported`] if the current platform has no compatible backend.
    pub fn new(service: impl Into<Cow<'static, str>>) -> Result<Self, LockboxError> {
        #[cfg(not(target_arch = "wasm32"))]
        use_native_store(false)?;

        #[cfg(target_arch = "wasm32")]
        use_named_store("sqlite").with_help(
            "In WASM environments, ensure the 'sqlite' store is initialized via virtual FS.",
        )?;

        Ok(Self { service: Arc::new(service.into()) })
    }

    /// Initializes a `Lockbox` instance using a specific store by name.
    ///
    /// See [`NAMED_STORES`] for a list of available backends.
    ///
    /// # Errors
    /// Returns [`LockboxError::InvalidStore`] if the provided `store_name` is unknown
    /// or not compiled into the binary.
    pub fn with_store(
        service: impl Into<Cow<'static, str>>,
        store_name: impl AsRef<str>,
    ) -> Result<Self, LockboxError> {
        use_named_store(store_name.as_ref())?;
        Ok(Self { service: Arc::new(service.into()) })
    }

    /// Internal helper to get the entry from the global default store.
    fn get_entry(&self, key: impl AsRef<str>) -> Result<keyring_core::Entry, LockboxError> {
        let store = get_default_store().ok_or_else(LockboxError::no_store)?;

        store.build(&self.service, key.as_ref(), None).map_err(LockboxError::from)
    }

    /// Saves a plain-text secret (password, token, etc.) to the store.
    ///
    /// If an entry with the same `key` already exists, it will be overwritten.
    ///
    /// # Errors
    /// Returns [`LockboxError::NoStore`] if no backend is initialized, or [`LockboxError::Core`]
    /// if the system store refuses the write operation (e.g., if the keychain is locked).
    pub fn set(&self, key: impl AsRef<str>, value: impl AsRef<str>) -> Result<(), LockboxError> {
        let entry = self.get_entry(key.as_ref())?;

        entry.set_password(value.as_ref()).map_err(LockboxError::from)
    }

    /// Retrieves a plain-text secret from the store.
    ///
    /// # Errors
    /// Returns an error if the key is not found or if the stored data is not valid UTF-8.
    pub fn get(&self, key: impl AsRef<str>) -> Result<String, LockboxError> {
        let entry = self.get_entry(key)?;

        entry.get_password().map_err(LockboxError::from)
    }

    /// Deletes a secret from the store.
    ///
    /// # Errors
    /// Returns an error if the deletion fails or if the entry does not exist.
    pub fn delete(&self, key: impl AsRef<str>) -> Result<(), LockboxError> {
        let entry = self.get_entry(key.as_ref())?;

        entry.delete_credential().map_err(LockboxError::from)
    }

    /// Serializes a value to JSON and saves it securely.
    ///
    /// This is useful for storing complex structures like session objects or configuration sets.
    ///
    /// # Errors
    /// Returns [`LockboxError::Serialization`] if the value cannot be serialized to JSON,
    /// in addition to standard storage errors.
    pub fn set_json<T: Serialize>(
        &self,
        key: impl AsRef<str>,
        value: &T,
    ) -> Result<(), LockboxError> {
        let json_string = serde_json::to_string(value)
            .with_details_fn(|| format!("Serializing JSON for key: {}", key.as_ref()))?;
        self.set(key.as_ref(), &json_string)
    }

    /// Retrieves a JSON string from the store and deserializes it into the target type.
    ///
    /// # Errors
    /// Returns [`LockboxError::Serialization`] if the retrieved string is not valid JSON
    /// or does not match the target structure.
    pub fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        key: impl AsRef<str>,
    ) -> Result<T, LockboxError> {
        let value = self.get(key.as_ref())?;
        serde_json::from_str(&value)
            .with_details_fn(|| format!("Deserializing JSON for key: {}", key.as_ref()))
    }
}

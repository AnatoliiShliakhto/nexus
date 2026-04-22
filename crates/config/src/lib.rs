//! # Nexus Config
//!
//! A production-grade, layered configuration system for Rust applications with a focus on
//! security and the **12-Factor App** methodology.
//!
//! ## Core Philosophies
//!
//! * **Fortress-First Security:** Secrets are never stored in plain text after initialization.
//!   They are discovered through secure channels (Docker/K8s secrets, Vault, or encrypted environment variables)
//!   and wrapped in `SecretString` to prevent accidental logging.
//! * **Typestate Safety:** The library uses the Typestate pattern to ensure that configuration
//!   cannot be accessed until it has been successfully loaded and validated.
//! * **Layered Configuration:** Powered by `config-rs`, it merges default files, environment-specific
//!   overrides (TOML, JSON, YAML), and system environment variables into a single unified state.
//!
//! ## Secret Discovery Strategy
//!
//! The `secret()` method follows a strict multi-stage discovery process:
//! 1.  **Orchestrator Secrets:** Checks `/run/secrets/<key>` (Standard for Docker Swarm and Kubernetes).
//! 2.  **Indirect Path:** Checks the environment variable `<KEY>_PATH` for a file location.
//! 3.  **Direct Environment:** Fallbacks to the environment variable `<KEY>` itself.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use nx_config::Config;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize, Clone)]
//! struct MyConfig {
//!     port: u16,
//!     database_url: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), nx_config::error::ConfigError> {
//!     // 1. Initialize and load from a directory (scans for default.toml, etc.)
//!     // 2. Environment variables with prefix NX__ will override file values.
//!     let config = Config::init::<MyConfig>()
//!         .load("config")
//!         .await?;
//!
//!     // 3. (Optional) Fetch additional secrets from HashiCorp Vault
//!     let config = config.fetch_vault("service/my-app").await?;
//!
//!     // 4. Access typed data
//!     let port = config.get().port;
//!
//!     // 5. Access sensitive secrets securely
//!     let api_key: String = config.secret("external_api_key")?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! * **Environment Overrides:** Use `NX__SECTION__KEY=value` to override `section.key` in your files.
//! * **Vault Integration:** Native support for KV2 engines with `AppRole` or Token authentication.
//! * **Zeroization:** Leverages the `secrecy` crate to ensure sensitive data is handled with care in memory.
mod environment;
pub mod error;
mod file;
mod vault;

use crate::environment::{env_secret, env_var};
use crate::error::ConfigError;
use crate::file::load_file;
use crate::private::{ConfigState, State};
use crate::vault::vault_variables;
use dashmap::DashMap;
use heck::ToSnakeCase;
pub use secrecy::zeroize::Zeroize;
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

// --- State ---
#[derive(Debug, Clone)]
pub struct NotInitialized;
#[derive(Debug, Clone)]
pub struct Initialized;

#[derive(Debug, Clone)]
pub struct NoConfig;
#[derive(Debug, Clone)]
pub struct WithConfig;

// --- Encapsulation ---
mod private {
    use crate::{Initialized, NoConfig, NotInitialized, WithConfig};

    pub trait State {}
    impl State for NotInitialized {}
    impl State for Initialized {}

    pub trait ConfigState {}
    impl ConfigState for NoConfig {}
    impl ConfigState for WithConfig {}
}

#[derive(Debug, Clone)]
pub struct Config<T = (), S = NotInitialized, C = NoConfig>
where
    T: DeserializeOwned + Debug + Clone + Send + Sync + 'static,
    S: State,
    C: ConfigState,
{
    pub(crate) inner: Arc<ConfigInner<T>>,
    _state: PhantomData<(S, C)>,
}

#[derive(Debug, Clone)]
pub struct ConfigInner<T>
where
    T: DeserializeOwned + Debug + Clone + Send + Sync + 'static,
{
    pub(crate) data: Option<T>,
    pub(crate) secrets: DashMap<String, SecretString>,
}

impl Config<(), NotInitialized, NoConfig> {
    /// Initializes the configuration container and loads the `.env` file.
    ///
    /// This serves as the starting point for a layered configuration. It uses `dotenvy`
    /// to load variables from a `.env` file into the environment.
    ///
    /// ### Examples
    /// ```rust,ignore
    /// use nx_config::Config;
    /// let builder = Config::init::<Settings>();
    /// ```
    #[must_use]
    pub fn init<T>() -> Config<T, Initialized, NoConfig>
    where
        T: DeserializeOwned + Debug + Clone + Send + Sync + 'static,
    {
        let _ = dotenvy::dotenv();
        let inner = ConfigInner { data: None, secrets: DashMap::new() };
        Config { inner: Arc::new(inner), _state: PhantomData }
    }

    /// Loads configuration data by merging file sources and environment variables.
    ///
    /// This method leverages the `config-rs` layered system. It attempts to read from
    /// the specified path (supporting TOML, JSON, YAML, etc.) and then overrides
    /// those values with environment variables prefixed with `NX__`.
    ///
    /// ### Errors
    /// * [`ConfigError::Load`]: If the configuration exists but is malformed or
    ///   cannot be deserialized into type `T`.
    /// * [`ConfigError::Io`]: If there is a critical filesystem error.
    ///
    /// ### Examples
    /// ```rust,ignore
    /// # async fn run() -> Result<(), nx_config::error::ConfigError> {
    /// // Will look for config.toml, config.json, etc., and merge with NX__ env vars
    /// let config = Config::init::<Settings>().load("config").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_file<T>(
        path: impl AsRef<Path> + Send,
    ) -> Result<Config<T, Initialized, WithConfig>, ConfigError>
    where
        T: DeserializeOwned + Debug + Clone + Send + Sync + 'static,
    {
        Self::init().load(path).await
    }
}

impl<T> Config<T, Initialized, NoConfig>
where
    T: DeserializeOwned + Debug + Clone + Send + Sync + 'static,
{
    /// Loads the configuration data from the specified path and transitions the state.
    ///
    /// ### Errors
    /// * [`ConfigError::Load`]: Deserialization failed or file format is unsupported.
    /// * [`ConfigError::Io`]: File could not be read.
    pub async fn load(
        mut self,
        path: impl AsRef<Path> + Send,
    ) -> Result<Config<T, Initialized, WithConfig>, ConfigError> {
        let data = load_file::<T>(path.as_ref()).await?;
        {
            let inner_mut = Arc::make_mut(&mut self.inner);
            inner_mut.data = Some(data);
        }
        Ok(Config { inner: self.inner, _state: PhantomData })
    }
}

impl<T> Config<T, Initialized, WithConfig>
where
    T: DeserializeOwned + Debug + Clone + Send + Sync + 'static,
{
    /// Returns a shared reference to the fully merged configuration data.
    ///
    /// ### Panics
    /// This method will panic if the data was never loaded. The Typestate pattern
    /// (`WithConfig` marker) usually prevents this at compile time.
    #[must_use]
    pub fn get(&self) -> &T {
        self.inner.data.as_ref().expect("Config data must be present in WithConfig state")
    }
}

impl<T, C> Config<T, Initialized, C>
where
    T: DeserializeOwned + Debug + Clone + Send + Sync + 'static,
    C: ConfigState,
{
    /// Retrieves and parses a single environment variable.
    ///
    /// Useful for ad-hoc configuration values that are not part of the main
    /// configuration struct.
    ///
    /// ### Errors
    /// * [`ConfigError::Env`]: If the key is missing from the environment.
    /// * [`ConfigError::Invalid`]: If the value cannot be parsed into type `U`.
    pub fn var<U>(&self, key: &str) -> Result<U, ConfigError>
    where
        U: FromStr + Debug,
        <U as FromStr>::Err: Display,
    {
        env_var(key)?.parse::<U>().map_err(|e| {
            ConfigError::invalid().with_details(format!("Failed to parse `{key}`: {e}"))
        })
    }

    /// Retrieves a secret value using the Fortress-First discovery strategy.
    ///
    /// It checks:
    /// 1. `/run/secrets/{snake_case_key}` (Docker/K8s)
    /// 2. The path defined in `{SHOUTY_KEY}_PATH`
    /// 3. The environment variable `{SHOUTY_KEY}` directly.
    ///
    /// Successfully retrieved secrets are stored in a `SecretString` to prevent
    /// accidental exposure in logs or traces.
    ///
    /// ### Errors
    /// * [`ConfigError::NotFound`]: If the secret cannot be found in any of the stages.
    /// * [`ConfigError::Io`]: If a secret file was found but is unreadable.
    pub fn secret<U>(&self, key: &str) -> Result<U, ConfigError>
    where
        U: FromStr + Debug,
        <U as FromStr>::Err: Display,
    {
        let key = key.to_snake_case();
        if let Some(secret) = self.inner.secrets.get(&key) {
            return secret.expose_secret().parse::<U>().map_err(|e| {
                ConfigError::invalid().with_details(format!("Failed to parse secret `{key}`: {e}"))
            });
        }

        let raw = env_secret(&key)?;
        self.inner.secrets.insert(key.clone(), SecretString::from(raw.clone()));
        raw.parse::<U>().map_err(|e| {
            ConfigError::invalid().with_details(format!("Failed to parse secret `{key}`: {e}"))
        })
    }

    /// Fetches secrets from `HashiCorp Vault KV2` and merges them into the internal cache.
    ///
    /// This method uses the following environment variables for configuration:
    /// * `VAULT_ADDR`: The URL of the Vault server (Required).
    /// * `VAULT_MOUNT`: The mount point of the KV2 engine (Defaults to `"secret"`).
    /// * `VAULT_TOKEN`: Static token for authentication.
    /// * `VAULT_ROLE_ID` & `VAULT_SECRET_ID`: Used for `AppRole` auth if no token is provided.
    /// * `VAULT_CERT`: Optional CA certificate for TLS connections.
    ///
    /// ### Errors
    /// * [`ConfigError::Vault`]: If communication with Vault fails or the client builder fails.
    /// * [`ConfigError::Invalid`]: If required credentials (Token or `AppRole`) are missing.
    /// * [`ConfigError::NotFound`]: If the specified `path` does not exist in the Vault mount.
    pub async fn fetch_vault(self, path: &str) -> Result<Self, ConfigError> {
        let variables = vault_variables(path).await?;
        for (key, value) in variables {
            self.inner.secrets.insert(key.to_snake_case(), SecretString::from(value));
        }
        Ok(self)
    }
}

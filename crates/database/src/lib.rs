//! # Nexus Database Client
//!
//! A high-performance, type-safe `SurrealDB` client specifically engineered for
//! `WebAssembly` (Wasm) environments using the Spin SDK.
//!
//! ## Core Philosophy
//! * **Type-State Safety**: Leveraging the Type-State pattern to prevent uninitialized
//!   database handles at compile time.
//! * **`FlatBuffers` Native**: Optimized for the binary `FlatBuffers` protocol to ensure
//!   minimal latency in high-load scenarios.
//! * **Fortress-First Security**: Built-in support for hardened identity management
//!   and secure session handling.
//!
//! ## Example
//! ```rust,ignore
//! use nx_database::Database;
//! use nx_database::error::DatabaseError;
//! use surrealdb_types::{SurrealValue, RecordId};
//!
//! #[derive(SurrealValue)]
//! struct User {
//!     id: RecordId,
//!     name: String,
//!     active: bool,
//! }
//!
//!
//! async fn example() -> Result<(), DatabaseError> {
//!     let db = Database::builder_from_env()?
//!         .credentials("admin", "password")?
//!         .init()
//!         .await?;
//!
//!     let response = db.query("SELECT * FROM user WHERE active = $active")
//!         .bind("active", true)
//!         .execute()
//!         .await?;
//!
//!     let users = response.last::<Vec<User>>().await?;
//!
//!     Ok(())
//! }
//! ```

use nx_http::url::Url;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

mod builder;
pub mod error;
mod query;
mod result;

use crate::builder::Configured;
pub use crate::builder::DatabaseBuilder;
use crate::error::DatabaseError;
use crate::query::Query;
pub use result::{QueryResult, ResultSegment};

/// Internal state for the database connection.
///
/// This struct holds the authentication token and target routing information
/// (Namespace/Database). It is wrapped in an `Arc` within the [`Database`]
/// handle to allow efficient cloning.
pub(crate) struct DatabaseInner {
    pub(crate) url: Url,
    pub(crate) token: String,
    pub(crate) namespace: String,
    pub(crate) database: String,
}

impl Debug for DatabaseInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            .field("url", &self.url.as_str())
            .field("namespace", &self.namespace)
            .field("database", &self.database)
            .finish_non_exhaustive()
    }
}

/// A thread-safe handle to a `SurrealDB` instance.
///
/// This struct is a lightweight wrapper around an `Arc<DatabaseInner>`.
/// Cloning a `Database` handle is an inexpensive operation and does not
/// re-initialize the connection.
#[derive(Clone)]
pub struct Database {
    pub(crate) inner: Arc<DatabaseInner>,
}

impl Debug for Database {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl Database {
    /// Initializes a [`DatabaseBuilder`] with explicit connection parameters.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::InvalidConfig`] if:
    /// * The provided `url` cannot be parsed.
    /// * Either `database` or `namespace` strings are empty.
    pub fn builder(
        url: impl AsRef<str>,
        database: impl Into<String>,
        namespace: impl Into<String>,
    ) -> Result<DatabaseBuilder<Configured>, DatabaseError> {
        DatabaseBuilder::new(url, database, namespace)
    }

    /// Creates a builder by reading `DATABASE_URL`, `DATABASE_NAMESPACE`, and `DATABASE_NAME`
    /// from the process environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::ConfigInvalid`] if any required environment
    /// variable is missing or malformed.
    pub fn builder_from_env() -> Result<DatabaseBuilder<Configured>, DatabaseError> {
        DatabaseBuilder::from_env()
    }

    /// Creates a builder using the `Spin` configuration variables.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError`] if the Spin host fails to resolve the
    /// required configuration keys or if the values are invalid.
    pub async fn builder_from_vars() -> Result<DatabaseBuilder<Configured>, DatabaseError> {
        DatabaseBuilder::from_vars().await
    }

    /// Constructs a new SQL query for execution.
    ///
    /// Returns a [`Query`] object that supports parameter binding and
    /// asynchronous execution.
    ///
    /// # Arguments
    /// * `sql` - The SQL script to execute. Can be a static string or a `String`.
    pub fn query<S: Into<Cow<'static, str>>>(&self, sql: S) -> Query {
        Query::new(self, sql)
    }
}

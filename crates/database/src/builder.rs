use self::private::Sealed;
use crate::error::DatabaseError;
use crate::{Database, DatabaseInner};
use nx_http::spin_sdk::http::body::IncomingBodyExt;
use nx_http::spin_sdk::http::{Method, Request};
use nx_http::spin_tools::environment::get_spin_var;
use nx_http::url::Url;
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::Arc;
use surrealdb_types::SurrealValue;

#[derive(Debug)]
pub struct Empty;
#[derive(Debug)]
pub struct Configured;
#[derive(Debug)]
pub struct Ready;

pub(crate) mod private {
    pub(super) trait Sealed {}
}

impl Sealed for Empty {}
impl Sealed for Configured {}
impl Sealed for Ready {}

enum Auth {
    Token(String),
    Credentials { username: String, password: String },
}

impl std::fmt::Debug for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token(_) => write!(f, "Token(***)"),
            Self::Credentials { username, .. } => f
                .debug_struct("Credentials")
                .field("username", username)
                .field("password", &"********")
                .finish(),
        }
    }
}

/// A type-safe builder for initializing a `SurrealDB` connection handle.
///
/// This builder uses the Type-State pattern to ensure that a connection
/// can only be initialized once the mandatory configuration (URL, Namespace, Database)
/// and authentication (Token or Credentials) have been provided.
///
/// # States
/// * `Empty` - Initial state, requires basic connection parameters.
/// * `Configured` - Connection parameters set, requires authentication.
/// * `Ready` - All parameters provided, ready to perform the `init` handshake.
#[allow(private_bounds)]
#[derive(Debug)]
pub struct DatabaseBuilder<S: Sealed = Empty> {
    url: Url,
    database: String,
    namespace: String,
    auth: Option<Auth>,
    _state: PhantomData<S>,
}

impl DatabaseBuilder<Empty> {
    /// Creates a new builder instance with the specified connection parameters.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::ConfigInvalid`] if:
    /// * The URL string is malformed and cannot be parsed.
    /// * Either the `namespace` or `database` strings are empty.
    pub fn new(
        url: impl AsRef<str>,
        database: impl Into<String>,
        namespace: impl Into<String>,
    ) -> Result<DatabaseBuilder<Configured>, DatabaseError> {
        let mut url = url.as_ref().to_owned();
        if !url.ends_with('/') {
            url.push('/');
        }
        let url = url.parse::<Url>().map_err(|e| {
            DatabaseError::config_invalid().with_details(format!("Invalid database url: {e}"))
        })?;
        let database = database.into();
        let namespace = namespace.into();

        if namespace.is_empty() || database.is_empty() {
            return Err(DatabaseError::config_invalid()
                .with_details("Database and namespace cannot be empty"));
        }

        Ok(DatabaseBuilder { url, database, namespace, auth: None, _state: PhantomData })
    }

    /// Creates a builder by reading connection parameters from environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::ConfigInvalid`] if any of the required variables
    /// (`DATABASE_URL`, `DATABASE_NAMESPACE`, `DATABASE_NAME`) are missing or invalid.
    pub fn from_env() -> Result<DatabaseBuilder<Configured>, DatabaseError> {
        let url = std::env::var("DATABASE_URL").map_err(|_| {
            DatabaseError::config_invalid()
                .with_details("DATABASE_URL environment variable is not set")
        })?;
        let namespace = std::env::var("DATABASE_NAMESPACE").map_err(|_| {
            DatabaseError::config_invalid()
                .with_details("DATABASE_NAMESPACE environment variable is not set")
        })?;
        let database = std::env::var("DATABASE_NAME").map_err(|_| {
            DatabaseError::config_invalid()
                .with_details("DATABASE_NAME environment variable is not set")
        })?;
        Self::new(url, database, namespace)
    }

    /// Creates a builder using the `Spin` configuration provider.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError`] if the `Spin` host fails to provide the requested
    /// configuration variables or if the provided values are invalid.
    pub async fn from_vars() -> Result<DatabaseBuilder<Configured>, DatabaseError> {
        let url = get_spin_var("database_url", None).await?;
        let namespace = get_spin_var("database_namespace", None).await?;
        let database = get_spin_var("database_name", None).await?;
        Self::new(url, database, namespace)
    }
}

impl DatabaseBuilder<Configured> {
    /// Provides a static Bearer token for authentication.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::ConfigInvalid`] if the provided token string is empty.
    pub fn token(self, token: impl Into<String>) -> Result<DatabaseBuilder<Ready>, DatabaseError> {
        let token = token.into();
        if token.is_empty() {
            return Err(DatabaseError::config_invalid().with_details("Token cannot be empty"));
        }
        Ok(DatabaseBuilder {
            url: self.url,
            database: self.database,
            namespace: self.namespace,
            auth: Some(Auth::Token(token)),
            _state: PhantomData,
        })
    }

    /// Provides user credentials to be exchanged for a token during initialization.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::ConfigInvalid`] if either the `username` or
    /// `password` strings are empty.
    pub fn credentials(
        self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<DatabaseBuilder<Ready>, DatabaseError> {
        let username = username.into();
        let password = password.into();
        if username.is_empty() || password.is_empty() {
            return Err(DatabaseError::config_invalid()
                .with_details("Username and password cannot be empty"));
        }
        Ok(DatabaseBuilder {
            url: self.url,
            database: self.database,
            namespace: self.namespace,
            auth: Some(Auth::Credentials { username, password }),
            _state: PhantomData,
        })
    }
}

impl DatabaseBuilder<Ready> {
    /// Finalizes the configuration and initializes the `Database` handle.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError`] if:
    /// * Authentication credentials were not properly set.
    /// * The network request to the `/signin` endpoint fails.
    /// * The server returns an error status code.
    /// * The response body fails to decode from `FlatBuffers`.
    #[allow(clippy::future_not_send)]
    pub async fn init(self) -> Result<Database, DatabaseError> {
        let Self { url, database, namespace, auth, .. } = self;

        let token = match auth {
            Some(Auth::Token(token)) => token,
            Some(Auth::Credentials { username, password }) => {
                signin(url.clone(), namespace.clone(), database.clone(), username, password).await?
            },
            None => {
                return Err(DatabaseError::config_invalid()
                    .with_details("Authentication credentials are required"));
            },
        };

        let db = Database {
            inner: Arc::new(DatabaseInner {
                url,
                token: format!("Bearer {token}"),
                namespace,
                database,
            }),
        };

        Ok(db)
    }
}

#[derive(Debug, Serialize)]
struct SignInRequest {
    ns: String,
    db: String,
    user: String,
    pass: String,
}

#[derive(Debug, SurrealValue)]
struct SignInResponse {
    code: u16,
    details: String,
    token: String,
}

/// Performs an asynchronous authentication request against the `SurrealDB` `/signin` endpoint.
///
/// This function exchanges a username and password for a JWT token, scoped
/// to the specific namespace and database provided.
///
/// # Arguments
/// * `url` - The base URL of the `SurrealDB` instance.
/// * `ns` - Target namespace.
/// * `db` - Target database.
/// * `user` - Authentication username.
/// * `pass` - Authentication password.
///
/// # Returns
/// * `Ok(String)` - A valid JWT authentication token.
/// * `Err(DatabaseError)` - If the network request fails, the server returns an error
///   status, or the response body cannot be decoded.
///
/// # Security
/// This function uses `application/json` for the request payload and expects
/// `application/vnd.surrealdb.flatbuffers` for the response to ensure high-performance
/// deserialization.
#[allow(clippy::future_not_send)]
async fn signin(
    url: Url,
    ns: String,
    db: String,
    user: String,
    pass: String,
) -> Result<String, DatabaseError> {
    let payload = serde_json::to_string(&SignInRequest { ns, db, user, pass })?;
    let response = nx_http::spin_sdk::http::send(
        Request::builder()
            .method(Method::POST)
            .uri(url.join("signin")?.as_str())
            .header("content-type", "application/json")
            .header("accept", "application/vnd.surrealdb.flatbuffers")
            .body(payload)?,
    )
    .await?;
    let status = response.status().as_u16();
    let body = response.into_body().bytes().await?;
    if status.ge(&400) {
        return Err(DatabaseError::from(body.as_ref()));
    }
    let claims = surrealdb_types::decode::<SignInResponse>(body.as_ref())
        .map_err(|e| DatabaseError::flat_buffers().with_details(e.to_string()))?;
    Ok(claims.token)
}

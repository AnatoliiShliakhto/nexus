use crate::core::config::GatewayConfig;
use chrono::Duration;
use futures::{Stream, StreamExt};
use jsonwebtoken::{EncodingKey, encode};
use nx_config::error::ConfigError;
use nx_error::prelude::*;
use serde::Serialize;
use std::ops::Deref;
use std::sync::Arc;
use surrealdb::engine::any::{Any, connect};
use surrealdb::method::{IntoVariables, Query};
use surrealdb::opt::auth::Root;
use surrealdb::types::SurrealValue;
use surrealdb::{IndexedResults, Notification, Surreal};

#[error]
pub enum DatabaseError {
    #[transparent(nx_config::error::ConfigError)]
    Config,

    #[error(
        message = "Data validation failed",
        status = ErrorStatus::BadRequest,
        code = "DB_QUERY_VALIDATION_FAILED",
    )]
    Validation,

    #[error(
        message = "A record with this unique identifier already exists",
        status = ErrorStatus::Conflict,
        code = "DB_QUERY_CONSTRAINT_VIOLATION",
    )]
    Conflict,

    #[error(
        message = "The requested record was not found",
        status = ErrorStatus::NotFound,
        code = "DB_QUERY_RECORD_NOT_FOUND",
    )]
    NotFound,

    #[error(
        message = "Database access denied",
        status = ErrorStatus::Unauthorized,
        code = "DB_INFRA_AUTH_REJECTED",
    )]
    Auth,

    #[error(
        message = "Database connectivity issue",
        status = ErrorStatus::BadGateway,
        code = "DB_INFRA_CONNECTION_FAILED",
    )]
    Connection,

    #[error(
        message = "Query executed with internal statement errors",
        status = ErrorStatus::InternalServerError,
        code = "DB_SYS_STATEMENT_ERROR",
    )]
    Execution,

    #[error(
        message = "Failed to deserialize result at specific index",
        status = ErrorStatus::InternalServerError,
        code = "DB_RESULT_MAPPING_FAILED",
    )]
    Mapping,

    #[error(
        message = "An internal database engine failure occurred",
        status = ErrorStatus::InternalServerError,
        code = "DB_SYS_SURREAL_ENGINE_ERROR",
        source = surrealdb::Error,
    )]
    Engine,

    #[error(
        message = "Failed to sign database access token",
        status = ErrorStatus::InternalServerError,
        code = "DB_AUTH_TOKEN_SIGNING_FAILED",
        source = jsonwebtoken::errors::Error,
    )]
    Token,

    #[error(
        message = "An internal database engine failure occurred",
        status = ErrorStatus::InternalServerError,
        code = "DB_SYS_SURREAL_ENGINE_ERROR",
    )]
    Internal,
}

impl DatabaseError {
    pub(crate) fn emit(&self) {
        tracing::error!(
            code = %self.code(),
            details = %self.details().as_deref().unwrap_or(""),
            "{}", self.message(),
        );
    }
}

#[derive(Debug, Serialize)]
struct DatabaseClaims<'a> {
    pub ns: &'a str,
    pub db: &'a str,
    pub ac: &'static str,
    pub id: String,
    pub exp: i64,
}

#[derive(SurrealValue)]
pub(crate) struct EmptyVars;

// --- Database Wrapper ---

#[derive(Debug, Clone)]
pub(crate) struct Database {
    inner: Arc<DatabaseInner>,
}

#[derive(Debug, Clone)]
struct DatabaseInner {
    database: Surreal<Any>,
    namespace: String,
    name: String,
    token_expiry: Duration,
    token_key: EncodingKey,
}

impl Deref for Database {
    type Target = Surreal<Any>;

    fn deref(&self) -> &Self::Target {
        &self.inner.database
    }
}

impl Database {
    #[tracing::instrument(skip(config), name = "db_init")]
    pub(crate) async fn init(config: GatewayConfig) -> Result<Self, DatabaseError> {
        tracing::info!("Initializing database connection...");

        let db_cfg = config.get().database.clone();
        if db_cfg.url.is_empty() || db_cfg.namespace.is_empty() || db_cfg.name.is_empty() {
            return Err(DatabaseError::from(
                ConfigError::not_found().with_message("Incomplete database configuration"),
            ));
        }
        let token_expiry = Duration::seconds(
            (config.get().security.identity.session.access_token_ttl_sec + 30) as i64,
        );

        let vault = config.fetch_vault("nexus/database").await?;
        let username = vault.secret::<String>("database_user")?;
        let password = vault.secret::<String>("database_pass")?;
        let private_key = vault.secret::<String>("database_private_key")?;
        let token_key = EncodingKey::from_ed_pem(private_key.as_bytes()).map_err(|e| {
            DatabaseError::internal()
                .with_message("Failed to initialize token signing key")
                .with_details(format!("PEM error: {e}"))
        })?;

        let db = connect(&db_cfg.url)
            .with_capacity(1024)
            .await
            .map_err(|e| DatabaseError::connection().with_details(e.to_string()))?;

        let mut backoff = std::time::Duration::from_millis(500);
        const MAX_ATTEMPTS: u32 = 5;

        for attempt in 1..=MAX_ATTEMPTS {
            match db.health().await {
                Ok(_) => break,
                Err(e) if attempt == MAX_ATTEMPTS => {
                    tracing::error!(code = "DB_SYS_STATEMENT_ERROR", details = %e, "Database cluster is unreachable after maximum attempts");
                    return Err(DatabaseError::connection().with_message("Cluster unreachable"));
                },
                Err(e) => {
                    tracing::warn!(attempt, ?backoff, error = %e, "Database not ready, retrying...");
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                },
            }
        }

        db.signin(Root { username, password })
            .await
            .map_err(|e| DatabaseError::auth().with_details(e.to_string()))?;

        db.use_ns(&db_cfg.namespace)
            .use_db(&db_cfg.name)
            .await
            .map_err(|e| DatabaseError::connection().with_details(e.to_string()))?;

        prerequisites(&db, &vault).await?;

        let version = db.version().await.map_or_else(|_| "unknown".to_owned(), |v| v.to_string());
        tracing::info!(namespace = %db_cfg.namespace, database = %db_cfg.name, %version, "SurrealDB connection established");

        Ok(Self {
            inner: Arc::new(DatabaseInner {
                database: db,
                namespace: db_cfg.namespace.clone(),
                name: db_cfg.name.clone(),
                token_expiry,
                token_key,
            }),
        })
    }

    pub(crate) fn query<'a>(&'a self, sql: &'a str) -> QueryBuilder<'a> {
        QueryBuilder { inner: self.inner.database.query(sql), sql }
    }

    #[tracing::instrument(skip(self), name = "db_gen_token")]
    pub(crate) fn generate_token(&self, account: &str) -> Result<String, DatabaseError> {
        let claims = DatabaseClaims {
            ns: &self.inner.namespace,
            db: &self.inner.name,
            ac: "account",
            id: format!("account:{account}"),
            exp: (chrono::Utc::now() + self.inner.token_expiry).timestamp(),
        };

        encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::EdDSA),
            &claims,
            &self.inner.token_key,
        )
        .map_err(DatabaseError::from)
    }
}

// --- Query Result ---

#[derive(Debug)]
pub(crate) struct QueryResult {
    inner: IndexedResults,
}

impl QueryResult {
    pub(crate) fn at<T: SurrealValue>(&mut self, index: usize) -> Result<T, DatabaseError> {
        self.inner
            .take::<Option<T>>(index)
            .map_err(|e| DatabaseError::mapping().with_details(format!("Index {index}: {e}")))?
            .ok_or_else(|| {
                DatabaseError::not_found().with_details(format!("No record at index {index}"))
            })
    }

    pub(crate) fn many_at<T: SurrealValue>(
        &mut self,
        index: usize,
    ) -> Result<Vec<T>, DatabaseError> {
        self.inner
            .take::<Vec<T>>(index)
            .map_err(|_| DatabaseError::mapping().with_details(format!("Index {index}")))
    }
}

// --- Query Builder ---

pub(crate) struct QueryBuilder<'a> {
    inner: Query<'a, Any>,
    sql: &'a str,
}

impl<'a> QueryBuilder<'a> {
    pub(crate) fn bind<V: IntoVariables>(mut self, vars: V) -> Self {
        self.inner = self.inner.bind(vars);
        self
    }

    #[tracing::instrument(skip(self), name = "db_execute", fields(sql = %truncate_sql(self.sql, 100)))]
    pub(crate) async fn execute(self) -> Result<QueryResult, DatabaseError> {
        let response = self.inner.await?;

        let inner =
            response.check().map_err(|e| DatabaseError::execution().with_details(e.to_string()))?;

        Ok(QueryResult { inner })
    }

    #[tracing::instrument(skip(self), name = "db_subscribe", fields(sql = %truncate_sql(self.sql, 100)))]
    pub(crate) async fn subscribe<T>(
        self,
    ) -> Result<impl Stream<Item = Result<Notification<T>, DatabaseError>>, DatabaseError>
    where
        T: SurrealValue + Unpin + Send + Sync + 'static,
    {
        let mut response = self.execute().await?;

        let stream = response
            .inner
            .stream::<Notification<T>>(0)
            .map_err(|e| DatabaseError::mapping().with_details(e.to_string()))?;

        Ok(stream.map(|item| item.map_err(DatabaseError::from)))
    }
}

// --- Helpers ---

async fn prerequisites(db: &Surreal<Any>, cfg: &GatewayConfig) -> Result<(), DatabaseError> {
    let session_cfg = cfg.get().security.identity.session.clone();
    let public_key = cfg.secret::<String>("database_public_key")?;

    let init_sql = format!(
        r"
        DEFINE PARAM OVERWRITE $CONFIG_SESSION_TTL VALUE {}d;
        DEFINE PARAM OVERWRITE $CONFIG_GRACE_PERIOD VALUE {}s;
        DEFINE PARAM OVERWRITE $CONFIG_TOKEN_LEN VALUE {};
        DEFINE ACCESS OVERWRITE account ON DATABASE TYPE RECORD
            WITH JWT ALGORITHM EDDSA KEY $public_key
            DURATION FOR TOKEN {}s, FOR SESSION {}s;
    ",
        session_cfg.refresh_token_ttl_days,
        session_cfg.grace_period_sec,
        session_cfg.refresh_token_len,
        session_cfg.access_token_ttl_sec,
        session_cfg.access_token_ttl_sec + 60
    );

    db.query(&init_sql).bind(("public_key", public_key)).await?.check()?;

    tracing::info!("Database prerequisites initialized successfully");
    Ok(())
}

fn truncate_sql(sql: &str, max_len: usize) -> &str {
    if sql.len() > max_len { &sql[..max_len] } else { sql }
}

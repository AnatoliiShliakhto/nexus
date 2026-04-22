use crate::error::DatabaseError;
use crate::{Database, QueryResult};
use fxhash::FxHashMap;
use nx_http::spin_sdk::http::body::IncomingBodyExt;
use nx_http::spin_sdk::http::{Method, Request as SpinRequest};
use std::borrow::Cow;
use surrealdb_types::{SurrealValue, ToSql, Value};

/// A builder for constructing and executing `SurrealDB` SQL queries.
///
/// `Query` allows for parameterized SQL execution by binding variables
/// to the query context using the `LET` statement. This ensures
/// protection against SQL injection and provides a clean interface for
/// dynamic query building.
///
/// This struct implements a consuming builder pattern; calling [`execute`]
/// consumes the query and performs the asynchronous network request.
#[derive(Debug)]
pub struct Query {
    db: Database,
    sql: Cow<'static, str>,
    params: FxHashMap<&'static str, Value>,
    error: Option<DatabaseError>,
}

impl Query {
    pub(crate) fn new<S>(db: &Database, sql: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self { db: db.clone(), sql: sql.into(), params: FxHashMap::default(), error: None }
    }

    /// Binds a single parameter to the query context.
    ///
    /// The parameter will be injected into the `SurrealDB` script as a
    /// `LET $name = <value>;` statement.
    ///
    /// # Arguments:
    /// * `name` - The variable name (without the `$` prefix).
    /// * `value` - Any type that implements [`SurrealValue`].
    pub fn bind<V: SurrealValue>(mut self, name: &'static str, value: V) -> Self {
        self.params.insert(name, value.into_value());
        self
    }

    /// Binds multiple parameters to the query context from an iterator.
    ///
    /// Useful for binding collections or maps of variables in a single call.
    pub fn bind_many<V, I>(mut self, params: I) -> Self
    where
        V: SurrealValue,
        I: IntoIterator<Item = (&'static str, V)>,
    {
        self.params.extend(params.into_iter().map(|(k, v)| (k, v.into_value())));
        self
    }

    /// Executes the query against the configured `SurrealDB` instance.
    ///
    /// This method performs the following steps:
    /// 1. Serializes all bound parameters into `LET` statements using [`ToSql`].
    /// 2. Appends the main SQL script.
    /// 3. Dispatches an asynchronous HTTP POST request to the `/sql` endpoint.
    /// 4. Validates the response status code.
    ///
    /// # Returns
    /// * `Ok(QueryResult)` - The raw `SurrealDB` response, ready for decoding.
    /// * `Err(DatabaseError)` - If a transport error occurs, the server
    ///   returns a 4xx/5xx status, or if a deferred configuration error was encountered.
    ///
    /// # Errors
    /// * [`DatabaseError::FromResponse`] - Triggered if the database returns a status code ≥ 400.
    /// * [`DatabaseError::UrlInvalid`] - Triggered if the connection URL is invalid.
    ///
    /// # Implementation Details
    /// Uses `application/vnd.surrealdb.flatbuffers` as the acceptance header to
    /// minimize parsing overhead within the Spin environment.
    #[allow(clippy::future_not_send)]
    pub async fn execute(self) -> Result<QueryResult, DatabaseError> {
        if let Some(err) = self.error {
            return Err(err);
        }

        let mut script = String::with_capacity(self.sql.len() + (self.params.len() * 64));

        for (name, value) in &self.params {
            script.push_str("LET $");
            script.push_str(name);
            script.push_str(" = ");

            script.push_str(&value.to_sql());
            script.push_str(";\n");
        }

        script.push_str(&self.sql);

        let res = nx_http::spin_sdk::http::send(
            SpinRequest::builder()
                .method(Method::POST)
                .uri(self.db.inner.url.join("sql")?.as_str())
                .header("content-type", "text/plain; charset=utf-8")
                .header("accept", "application/vnd.surrealdb.flatbuffers")
                .header("authorization", &self.db.inner.token)
                .header("surreal-ns", &self.db.inner.namespace)
                .header("surreal-db", &self.db.inner.database)
                .body(script)?,
        )
        .await?;

        if res.status().as_u16().ge(&400) {
            let err_body = res.into_body().bytes().await?;
            return Err(DatabaseError::from(err_body.as_ref()));
        }
        QueryResult::new(res).await
    }
}

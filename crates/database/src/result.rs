use crate::error::{DatabaseError, DatabaseErrorExt};
use heck::ToTitleCase;
use nx_http::bytes::Bytes;
use nx_http::spin_sdk::http::Response;
use nx_http::spin_sdk::http::body::IncomingBodyExt;
use serde::Deserialize;
use std::sync::{Arc, OnceLock};
use surrealdb_types::{SurrealValue, Value};

/// A high-level wrapper around a `SurrealDB` HTTP response result.
///
/// This struct manages the raw [`Bytes`] and provides methods for lazy
/// decoding and segment traversal. It specifically handles the multi-statement
/// nature of `SurrealDB` queries, where a single request may return multiple
/// result sets (segments).
///
/// Internal segments are cached using a [`OnceLock`] to ensure that expensive
/// `FlatBuffers` decoding is performed only once.
pub struct QueryResult {
    inner: Arc<Bytes>,
    segments: Arc<OnceLock<Vec<ResultSegment>>>,
}

impl std::fmt::Debug for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryResult")
            .field("inner", &"(omitted for security reasons)")
            .field("segments", &self.segments)
            .finish()
    }
}

impl QueryResult {
    pub(crate) async fn new(response: Response) -> Result<Self, DatabaseError> {
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        if !content_type.eq_ignore_ascii_case("application/vnd.surrealdb.flatbuffers") {
            return Err(DatabaseError::unsupported_content()
                .with_details_fn(|| format!("Unsupported content type: {content_type}")))
            .with_help("FlatBuffers is the only supported content type for SurrealDB responses");
        }
        let body = response.into_body().bytes().await?;

        Ok(Self { inner: Arc::new(body), segments: Arc::new(OnceLock::new()) })
    }

    pub(crate) fn decode<T: SurrealValue>(&self) -> Result<T, DatabaseError> {
        surrealdb_types::decode::<T>(&self.inner)
            .map_err(|e| DatabaseError::flat_buffers().with_details(e.to_string()))
    }

    fn resolve_segments(&self) -> Result<&Vec<ResultSegment>, DatabaseError> {
        if let Some(segments) = self.segments.get() {
            return Ok(segments);
        }
        let decoded = self.decode::<Vec<ResultSegment>>()?;
        let _ = self.segments.set(decoded);
        self.segments.get().ok_or_else(|| {
            DatabaseError::internal().with_details("Failed to access initialized segments")
        })
    }

    /// Returns the total number of segments in the response.
    #[must_use]
    pub fn len(&self) -> usize {
        self.resolve_segments().map_or(0, Vec::len)
    }

    /// Validates all response segments for database-level errors.
    ///
    /// Iterates through all statements in the batch and ensures their status is "OK".
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::Query`] if any segment in the response failed.
    /// The error includes the database's error message and the specific error category.
    pub fn check(&self) -> Result<&Self, DatabaseError> {
        let segments = self.resolve_segments()?;
        for segment in segments {
            if !segment.status.eq_ignore_ascii_case("OK") {
                let message = segment
                    .kind
                    .as_ref()
                    .map_or_else(|| "Query Error".to_owned(), |k| k.to_title_case());

                let details = segment
                    .result
                    .as_string()
                    .map_or_else(|| "Unknown database error".to_owned(), ToOwned::to_owned);

                return Err(DatabaseError::query().with_message(message).with_details(details));
            }
        }
        Ok(self)
    }

    /// Attempts to retrieve the last meaningful result from the response segments.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError`] if:
    /// * The response body cannot be decoded into segments.
    /// * Deserialization of the selected segment into type `T` fails.
    pub fn last<T: SurrealValue>(&self) -> Result<Option<T>, DatabaseError> {
        let segments = self.resolve_segments()?;

        let target = segments.iter().rev().find(|s| {
            if s.result.is_null() || s.result.is_none() {
                return false;
            }
            if let Some(arr) = s.result.as_array()
                && arr.is_empty()
            {
                return false;
            }
            if !s.status.eq_ignore_ascii_case("OK") {
                return false;
            }
            true
        });

        target.map_or_else(
            || Ok(None),
            |s| T::from_value(s.result.clone()).map(Some).map_err(DatabaseError::from),
        )
    }

    /// Extracts a specific result segment by its index.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError`] if:
    /// * The provided `index` is out of bounds.
    /// * The data in the specified segment cannot be mapped to type `T`.
    pub fn take<T: SurrealValue>(&self, index: usize) -> Result<T, DatabaseError> {
        let segments = self.resolve_segments()?;
        let segment = segments.get(index).ok_or_else(|| {
            DatabaseError::not_found().with_message(format!("Segment {index} not found"))
        })?;

        T::from_value(segment.result.clone()).map_err(DatabaseError::from)
    }

    /// Filters and collects all segments that can be successfully decoded into type `T`.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError`] if the underlying response body failed to decode
    /// into the internal segment structure.
    pub fn find_all<T: SurrealValue>(&self) -> Result<Vec<T>, DatabaseError> {
        let segments = self.resolve_segments()?;
        Ok(segments.iter().filter_map(|s| T::from_value(s.result.clone()).ok()).collect())
    }
}

/// Represents an individual result set within a multi-statement `SurrealDB` response result.
///
/// Each SQL statement in a batch query (separated by `;`) produces a corresponding
/// `ResultSegment`.
#[derive(Debug, SurrealValue, Deserialize)]
pub struct ResultSegment {
    /// The actual data payload of the segment.
    /// Can be a single value, an array of records, or `Null`.
    pub result: Value,
    #[serde(default)]
    /// The execution status of the statement (e.g., "OK", "ERR").
    pub status: String,
    #[serde(default)]
    /// The execution time of this specific statement.
    pub time: String,
    #[serde(rename = "type")]
    r#type: Value,
    /// The error category if the status is not "OK".
    #[serde(default)]
    pub kind: Option<String>,
}

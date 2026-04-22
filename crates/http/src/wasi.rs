//! This module provides standardized structures for returning JSON error responses
//! within the service. It ensures that errors follow a consistent format
//! across all API endpoints.

#![allow(clippy::panic)]

use bytes::Bytes;
use http::Response;
use std::borrow::Cow;

/// A standardized error structure serialized as JSON in the response body.
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    status: u16,
    code: Cow<'static, str>,
    message: Cow<'static, str>,
}

impl ErrorResponse {
    /// Creates a new `ErrorResponse` instance.
    ///
    /// # Arguments
    /// * `status` - Valid HTTP status code.
    /// * `code` - Error code. Accepts both `&'static str` and `String`.
    /// * `message` - The error details. Accepts both `&'static str` and `String`.
    pub fn new(
        status: u16,
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self { status, code: code.into(), message: message.into() }
    }

    /// Transforms the `ErrorResponse` into production-ready [`Response<Bytes>`].
    ///
    /// # Results
    /// * Returns an HTTP response with the specified status code.
    /// * If JSON serialization fails internally, an empty body is returned as a fallback.
    ///
    /// # Panics
    /// * This method will **panic** if the [`Response::builder`] fails to construct
    ///   the response (e.g., due to invalid header names or values).
    #[must_use]
    pub fn build(self) -> Response<Bytes> {
        let body = serde_json::to_vec(&self).unwrap_or_default();

        Response::builder()
            .status(self.status)
            .header("content-type", "application/json")
            .body(Bytes::from(body))
            .expect("Failed to build response")
    }
}

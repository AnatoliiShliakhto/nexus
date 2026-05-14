//! HTTP response extensions for the NEXUS Web Client.
//!
//! This module provides the [`ResponseExt`] trait to simplify asynchronous
//! JSON deserialization of responses within Dioxus applications.

use crate::error::WebClientError;
use serde::de::DeserializeOwned;

/// Extension trait for [`reqwest::Response`] providing high-level resolution methods.
pub trait ResponseExt {
    /// Resolves the response body into a specific type `T`.
    ///
    /// This method consumes the response, buffers the full body as bytes,
    /// and attempts to deserialize it from JSON.
    ///
    /// # Errors
    ///
    /// Returns a [`WebClientError`] if:
    /// - The network connection fails during body streaming.
    /// - The body cannot be deserialized into type `T`.
    fn resolve<T: DeserializeOwned>(self)
    -> impl Future<Output = Result<T, WebClientError>> + Send;
}

impl ResponseExt for reqwest::Response {
    async fn resolve<T: DeserializeOwned>(self) -> Result<T, WebClientError> {
        let bytes = self.bytes().await.map_err(WebClientError::from)?;
        serde_json::from_slice(&bytes).map_err(WebClientError::from)
    }
}

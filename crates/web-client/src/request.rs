//! HTTP request extensions for the NEXUS Web Client.
//!
//! This module provides a unified [`RequestBody`] enum and the [`IntoBody`] trait
//! to abstract over different content types, including JSON, forms, and multipart data.

use bytes::Bytes;
use fxhash::FxHashMap;
use serde::Serialize;
use serde_json::Value;
#[allow(clippy::disallowed_types)]
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

/// Represent high-level request body types supported by the system.
#[derive(Debug)]
pub enum RequestBody {
    /// A JSON payload represented as a [`Value`].
    Json(Value),
    /// Binary data with an associated MIME type.
    Bytes(String, Bytes),
    /// URL-encoded form data using a fast-hash map.
    Form(FxHashMap<String, String>),
    /// Complex multipart/form-data request.
    Multipart(MultipartRequest),
    /// No payload.
    Empty,
}

impl Clone for RequestBody {
    fn clone(&self) -> Self {
        match self {
            Self::Json(v) => Self::Json(v.clone()),
            Self::Bytes(t, b) => Self::Bytes(t.clone(), b.clone()),
            Self::Form(f) => Self::Form(f.clone()),
            Self::Multipart(m) => Self::Multipart(m.clone()),
            Self::Empty => Self::Empty,
        }
    }
}

/// A trait for types that can be converted into a [`RequestBody`].
pub trait IntoBody {
    /// Performs the conversion.
    fn into_body(self) -> RequestBody;
}

/// Wrapper for serializable types to be treated as JSON payloads.
#[derive(Debug)]
pub struct Json<T>(pub T);
impl<T: Serialize> IntoBody for Json<T> {
    fn into_body(self) -> RequestBody {
        RequestBody::Json(serde_json::to_value(&self.0).unwrap_or(Value::Null))
    }
}

impl IntoBody for (&str, Bytes) {
    fn into_body(self) -> RequestBody {
        RequestBody::Bytes(self.0.to_owned(), self.1)
    }
}

impl IntoBody for (&str, &[u8]) {
    fn into_body(self) -> RequestBody {
        RequestBody::Bytes(self.0.to_owned(), Bytes::copy_from_slice(self.1))
    }
}

impl IntoBody for (&str, Vec<u8>) {
    fn into_body(self) -> RequestBody {
        RequestBody::Bytes(self.0.to_owned(), Bytes::from(self.1))
    }
}

#[allow(clippy::disallowed_types)]
impl<S: std::hash::BuildHasher> IntoBody for HashMap<String, String, S> {
    fn into_body(self) -> RequestBody {
        RequestBody::Form(self.into_iter().collect())
    }
}

impl<const N: usize> IntoBody for [(&str, &str); N] {
    fn into_body(self) -> RequestBody {
        let mut map = FxHashMap::default();
        for (k, v) in self {
            map.insert(k.to_owned(), v.to_owned());
        }
        RequestBody::Form(map)
    }
}

impl IntoBody for () {
    fn into_body(self) -> RequestBody {
        RequestBody::Empty
    }
}

// --- Multipart ---

/// A container for `multipart/form-data` requests.
///
/// Supports both text fields and binary file uploads.
///
/// # Example
/// ```rust,ignore
/// let req = MultipartRequest::new()
///     .with_field("description", "system logs")
///     .with_file("log", "auth.log", bytes::Bytes::from("..."));
/// ```
#[derive(Clone)]
pub struct MultipartRequest {
    /// Simple key-value text fields.
    pub fields: FxHashMap<String, String>,
    /// Files indexed by field name, containing (filename, data).
    pub files: FxHashMap<String, (String, Bytes)>,
}

impl Debug for MultipartRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultipartRequest")
            .field("fields", &self.fields)
            .field("files", &self.files.keys())
            .finish()
    }
}

impl MultipartRequest {
    /// Creates a new, empty `MultipartRequest`.
    #[must_use]
    pub fn new() -> Self {
        Self { fields: FxHashMap::default(), files: FxHashMap::default() }
    }

    /// Adds a text field to the request.
    #[must_use]
    pub fn with_field(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(name.into(), value.into());
        self
    }

    /// Adds a file to the request.
    #[must_use]
    pub fn with_file(
        mut self,
        field: impl Into<String>,
        filename: impl Into<String>,
        data: Bytes,
    ) -> Self {
        self.files.insert(field.into(), (filename.into(), data));
        self
    }
}

impl Default for MultipartRequest {
    fn default() -> Self {
        Self::new()
    }
}

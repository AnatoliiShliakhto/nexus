//! Centralized error handling system.
//!
//! This crate provides a unified error model that combines:
//! - **ErrorStatus-based Typing**: Seamless compatibility with HTTP status codes.
//! - **Details Wrapping**: The ability to attach high-level descriptions as errors bubbles up.
//! - **Cause Chains**: Tools for debugging and iterating through nested error sources.
//! - **Procedural Macros**: Automatic boilerplate generation via the `#[error]` attribute.

mod status;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{ErrorMetadata, ErrorMetadataExt, ErrorStatus, error};
}

#[cfg(feature = "json")]
use serde::Serialize;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;

pub use nx_error_macros::error;
pub use status::ErrorStatus;

// --- Error Metadata Trait ---

/// A trait providing extended, structured metadata for errors in the Nexus ecosystem.
///
/// While the standard [`Error`] trait focuses on the error message and cause,
/// `ErrorMetadata` provides a machine-readable layer including status codes, unique error
/// slugs, and troubleshooting help. This is the foundation for the "Fortress-First"
/// observability pattern used across the platform.
///
/// # Implementing `ErrorMetadata`
///
/// Most users should not implement this trait manually. Instead, use the `#[nx_error::error]`
/// procedural macro to generate optimized implementations.
///
/// # Shadowing Note
/// This trait defines `error_source()`. We avoid using the name `source()` to prevent
/// ambiguity with [`Error::source`], allowing both to coexist while providing
/// specialized metadata traversal.
pub trait ErrorMetadata: fmt::Display + fmt::Debug + Error + 'static {
    /// Returns the HTTP-compatible status of the error.
    /// Defaults to `ErrorStatus::InternalServerError` (500).
    fn status(&self) -> ErrorStatus {
        ErrorStatus::InternalServerError
    }

    /// Returns a machine-readable unique identifier for the error variant.
    /// Typically formatted in `SHOUTY_SNAKE_CASE` (e.g., `DATABASE_CONNECTION_FAILED`).
    fn code(&self) -> Cow<'static, str> {
        Cow::Borrowed("INTERNAL_SERVER_ERROR")
    }

    /// Returns the primary human-readable description of the error.
    /// Uses [`Cow<'static, str>`] to avoid heap allocation for static strings.
    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("Internal Server Error")
    }

    /// Returns high-level operational context attached to the error.
    /// This is used to "wrap" the error with dynamic data (e.g., "Failed for user ID: 123").
    fn details(&self) -> Option<Cow<'static, str>> {
        None
    }

    /// Provides access to the next error in the chain if it also implements `ErrorMetadata`.
    /// This allows for recursive metadata resolution (e.g., in `#[transparent]` delegation).
    fn error_source(&self) -> Option<&(dyn ErrorMetadata + 'static)> {
        None
    }

    /// Returns the target component or subsystem where the error originated.
    /// Useful for telemetry filtering and distributed tracing.
    fn target(&self) -> &'static str {
        "UNKNOWN_TARGET"
    }

    /// Returns actionable troubleshooting advice for the end-user or operator.
    /// Example: "Ensure the disk is not read-only."
    fn help(&self) -> Option<Cow<'static, str>> {
        None
    }
}

// --- Error Extension Trait ---

#[cfg(feature = "json")]
const DEFAULT_FALLBACK: &str = r#"{"status":500,"code":"INTERNAL_SERVER_ERROR","message":"An unexpected error occurred during response formatting"}"#;

/// An extension trait providing high-level utilities for all types implementing [`ErrorMetadata`].
///
/// This trait is automatically implemented for any type that satisfies the `ErrorMetadata` bounds.
/// It provides the primary entry points for reporting, serialization, and chain inspection.
pub trait ErrorMetadataExt: ErrorMetadata {
    /// Returns an iterator over the entire error cause chain.
    ///
    /// The iterator yields the current error first, then recursively follows `error_source()`.
    ///
    /// # Example
    /// ```rust,ignore
    /// for layer in err.chain() {
    ///     println!("Level: {}, Code: {}", layer.status(), layer.code());
    /// }
    /// ```
    fn chain(&self) -> ErrorChain<'_>
    where
        Self: Sized + 'static,
    {
        ErrorChain { current: Some(self) }
    }

    /// Wraps the error in a [`DetailedReport`] for pretty-printing.
    ///
    /// The report uses a tree-like ASCII structure to visualize the message,
    /// status, causes, details, and help information.
    ///
    /// # Example
    /// ```rust,ignore
    /// println!("{}", err.report());
    /// ```
    fn report(&self) -> DetailedReport<'_, Self> {
        DetailedReport(self)
    }

    /// Converts the error metadata into a simple serializable structure.
    /// Only includes `status`, `code`, and `message`.
    #[cfg(feature = "json")]
    fn to_serializable(&self) -> SerializableReport {
        SerializableReport::from_metadata(self)
    }

    /// Serializes the error to a basic JSON string.
    /// If serialization fails, returns a hardcoded fallback JSON.
    #[cfg(feature = "json")]
    fn to_json_string(&self) -> String {
        let report = self.to_serializable();
        serde_json::to_string(&report).unwrap_or_else(|_| {
            #[derive(Serialize)]
            struct Fallback<'a> {
                status: u16,
                code: &'a str,
                message: &'a str,
            }
            serde_json::to_string(&Fallback {
                status: report.status,
                code: &report.code,
                message: &report.message,
            })
            .unwrap_or_else(|_| DEFAULT_FALLBACK.to_owned())
        })
    }

    /// Converts the error into a detailed serializable structure,
    /// including the full cause chain, target, and details.
    #[cfg(feature = "json")]
    fn to_detailed_serializable(&self) -> DetailedSerializableReport {
        DetailedSerializableReport::from_metadata(self)
    }

    /// Serializes the error to a detailed JSON string, including the cause chain.
    /// Ideal for internal logging or telemetry collectors like Grafana Loki.
    #[cfg(feature = "json")]
    fn to_detailed_json_string(&self) -> String {
        let report = self.to_detailed_serializable();
        serde_json::to_string(&report).unwrap_or_else(|_| {
            #[derive(Serialize)]
            struct Fallback<'a> {
                status: u16,
                code: &'a str,
                message: &'a str,
                target: &'a str,
                details: Option<&'a str>,
                chain: &'a [String],
            }
            serde_json::to_string(&Fallback {
                status: report.status,
                code: &report.code,
                message: &report.message,
                target: report.target,
                details: report.details.as_deref(),
                chain: report.chain.as_slice(),
            })
            .unwrap_or_else(|_| DEFAULT_FALLBACK.to_owned())
        })
    }
}

impl<T: ErrorMetadata + ?Sized> ErrorMetadataExt for T {}
impl Error for Box<dyn ErrorMetadata + Send + Sync> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Error::source(self.as_ref())
    }
}

// --- Helpers ---

/// Centralized logic for extracting and deduplicating the cause chain.
fn collect_causes(start_source: Option<&(dyn ErrorMetadata + 'static)>) -> Vec<String> {
    let mut seen = fxhash::FxHashSet::default();
    std::iter::successors(start_source, |&e| e.error_source())
        .map(ToString::to_string)
        .filter(|msg| !msg.is_empty() && seen.insert(msg.clone()))
        .collect()
}

// --- Error chain ---

#[derive(Debug)]
pub struct ErrorChain<'a> {
    current: Option<&'a (dyn ErrorMetadata + 'static)>,
}

impl<'a> Iterator for ErrorChain<'a> {
    type Item = &'a (dyn ErrorMetadata + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = current.error_source();
        Some(current)
    }
}

// --- Error reporting ---

#[derive(Debug)]
pub struct DetailedReport<'a, E: ?Sized>(pub &'a E);

impl<E> fmt::Display for DetailedReport<'_, E>
where
    E: ErrorMetadata + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = self.0;

        writeln!(f, "  × [{}]: {}", err.code(), err.message())?;
        writeln!(
            f,
            "    Status: {} {} | Target: {}",
            err.status().as_u16(),
            err.status().as_str(),
            err.target()
        )?;

        let all_causes = collect_causes(err.error_source());

        let has_chain = !all_causes.is_empty();
        let has_details = err.details().is_some();
        let has_help = err.help().is_some();

        if has_chain || has_details || has_help {
            writeln!(f, "  │")?;
        }

        if has_chain {
            let is_last_section = !has_details && !has_help;
            let section_prefix = if is_last_section { "  ╰─" } else { "  ├─" };
            writeln!(f, "{section_prefix} Caused by:")?;

            let pipe = if is_last_section { "    " } else { "  │ " };
            for (i, cause) in all_causes.iter().enumerate() {
                let mut lines = cause.lines();
                if let Some(first_line) = lines.next() {
                    writeln!(f, "{pipe} {}: {first_line}", i + 1)?;
                }
                for line in lines {
                    writeln!(f, "{pipe}    {line}")?;
                }
            }

            if !is_last_section {
                writeln!(f, "  │")?;
            }
        }

        if let Some(details) = err.details() {
            let is_last_section = !has_help;
            let section_prefix = if is_last_section { "  ╰─" } else { "  ├─" };
            writeln!(f, "{section_prefix} Details:")?;

            let pipe = if is_last_section { "    " } else { "  │ " };
            for line in details.lines() {
                writeln!(f, "{pipe} {line}")?;
            }

            if !is_last_section {
                writeln!(f, "  │")?;
            }
        }

        if let Some(help) = err.help() {
            writeln!(f, "  ╰─ Help: {help}")?;
        }

        Ok(())
    }
}

// --- Serialization ---

#[cfg(feature = "json")]
#[derive(Debug, Serialize)]
pub struct SerializableReport {
    status: u16,
    code: String,
    message: String,
}

#[cfg(feature = "json")]
impl SerializableReport {
    pub fn from_metadata<E: ErrorMetadata + ?Sized>(err: &E) -> Self {
        Self {
            status: err.status().as_u16(),
            code: err.code().into_owned(),
            message: err.message().into_owned(),
        }
    }
}

#[cfg(feature = "json")]
#[derive(Debug, Serialize)]
pub struct DetailedSerializableReport {
    status: u16,
    code: String,
    message: String,
    target: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    chain: Vec<String>,
}

#[cfg(feature = "json")]
impl DetailedSerializableReport {
    pub fn from_metadata<E: ErrorMetadata + ?Sized>(err: &E) -> Self {
        Self {
            status: err.status().as_u16(),
            code: err.code().into_owned(),
            message: err.message().into_owned(),
            target: err.target(),
            details: err.details().map(Cow::into_owned),
            chain: collect_causes(err.error_source()),
        }
    }
}

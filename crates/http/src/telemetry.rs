//! # HTTP Telemetry and Observability Module
//!
//! This module provides a robust infrastructure for tracing and logging. It integrates `tracing-subscriber` with
//! custom filtering and HTTP header context extraction.
//!
//! ## Key Features
//! - **Flexible Initialization**: Supports JSON and console output based on environment variables.
//! - **Field Filtering**: Automatically redacts sensitive or noisy fields (e.g., passwords, backtraces).

pub use nx_logger::{LevelFilter, Logger};
use std::sync::OnceLock;

static TELEMETRY_INIT: OnceLock<Logger> = OnceLock::new();

/// Initializes the global telemetry subscriber.
///
/// Configuration is driven by the following environment variables:
/// - `LOG_JSON`: Set to `1` or `true` for structured JSON output (default: true).
/// - `LOG_ANSI`: Enables colored terminal output for non-JSON logs (default: false).
/// - `LOG_SPANS`: Includes a list of active spans in JSON logs (default: false).
/// - `LOG_IGNORE_FIELDS`: Comma-separated list of fields to hide from logs.
/// - `RUST_LOG`: Standard env filter directive (default: info).
///
/// If initialization fails, an error is printed to `stderr`.
///
/// # Panics
///
/// Panics if the logger initialization fails.
pub fn init() -> &'static Logger {
    TELEMETRY_INIT.get_or_init(|| {
        Logger::builder()
            .name(env!("CARGO_PKG_NAME"))
            .console(true)
            .init()
            .expect("Failed to initialize `Logger` for telemetry.")
    })
}

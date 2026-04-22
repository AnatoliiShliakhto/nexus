//! # Nexus Logger
//!
//! A centralized logging and observability utility for the Nexus.
//! It provides a unified way to configure console and file logging with
//! rotation, non-blocking I/O, and environment-based filtering.
//!
//! ## Core Features
//! - **Typestate Builder**: Prevents misconfiguration at compile time.
//! - **Non-blocking I/O**: High-performance logging using background workers (critical for Axum/Tokio).
//! - **Cloud-Native Config**: Overrides configuration via environment variables (`LOG_JSON`, `LOG_ANSI`).
//! - **Field Redaction**: Automatically filters sensitive fields like `password`.
//! - **WASM Compatibility**: Conditional compilation for environments without file systems.
//!
//! ## Example
//!
//! ```rust
//! use nx_logger::{Logger, LevelFilter};
//!
//! let _logger = Logger::builder()
//!     .name("nx-gateway")
//!     .console(true)
//!     .level(LevelFilter::DEBUG)
//!     .init()
//!     .unwrap();
//! ```

#![allow(unused_crate_dependencies)]
pub mod error;
#[cfg(feature = "opentelemetry-otlp")]
pub mod otlp;

use crate::error::LoggerError;
use private::Sealed;
use std::path::PathBuf;
use tracing::field::Field;
use tracing_subscriber::fmt::{FormatFields, layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

#[cfg(not(target_arch = "wasm32"))]
use crate::error::LoggerErrorExt;
#[cfg(not(target_arch = "wasm32"))]
use tracing_appender::non_blocking::WorkerGuard;

#[cfg(feature = "opentelemetry-otlp")]
use crate::otlp::{OpenTelemetryGuard, init_otlp_pipeline};
#[cfg(feature = "opentelemetry")]
use opentelemetry::global;
#[cfg(feature = "opentelemetry")]
use opentelemetry_sdk::propagation::TraceContextPropagator;
pub use tracing::level_filters::LevelFilter;

#[cfg(not(target_arch = "wasm32"))]
pub use tracing_appender::rolling::Rotation;

use tracing_subscriber::field::{MakeVisitor, Visit};
use tracing_subscriber::fmt::format::{DefaultFields, Writer};

#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_MAX_FILES: usize = 10;
#[cfg(not(target_arch = "wasm32"))]
const LOG_FILE_SUFFIX: &str = "log";

/// Internal configuration for the logger.
#[derive(Debug)]
pub struct LoggerConfig {
    console: bool,
    path: Option<PathBuf>,
    level: LevelFilter,
    #[cfg(not(target_arch = "wasm32"))]
    rotation: Rotation,
    #[cfg(not(target_arch = "wasm32"))]
    max_files: usize,
    json: bool,
    env_filter: Option<String>,
    #[cfg(feature = "opentelemetry")]
    opentelemetry: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            console: true,
            path: None,
            level: LevelFilter::INFO,
            #[cfg(not(target_arch = "wasm32"))]
            rotation: Rotation::DAILY,
            #[cfg(not(target_arch = "wasm32"))]
            max_files: DEFAULT_MAX_FILES,
            json: false,
            env_filter: None,
            #[cfg(feature = "opentelemetry")]
            opentelemetry: false,
        }
    }
}

// --- Builder Typestates ---

#[derive(Debug)]
pub struct NoName;
#[derive(Debug)]
pub struct WithName(String);
#[derive(Debug)]
pub struct NoFile;
#[derive(Debug)]
pub struct WithFile;

mod private {
    pub trait Sealed {}
}
impl Sealed for NoName {}
impl Sealed for WithName {}
impl Sealed for NoFile {}
impl Sealed for WithFile {}

/// A builder for configuring and initializing the global tracing subscriber.
#[derive(Debug)]
pub struct LoggerBuilder<N: Sealed = NoName, F: Sealed = NoFile> {
    config: LoggerConfig,
    name: N,
    file_state: std::marker::PhantomData<F>,
}

impl<F: Sealed> LoggerBuilder<NoName, F> {
    pub fn name(self, name: impl Into<String>) -> LoggerBuilder<WithName, F> {
        LoggerBuilder {
            name: WithName(name.into()),
            config: self.config,
            file_state: std::marker::PhantomData,
        }
    }
}

impl LoggerBuilder<WithName, WithFile> {
    /// Sets the maximum number of log files to retain.
    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub const fn max_files(mut self, max: usize) -> Self {
        {
            self.config.max_files = max;
        }
        self
    }

    /// Sets the log rotation frequency (Daily, Hourly, etc.).
    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub const fn rotation(mut self, rotation: Rotation) -> Self {
        {
            self.config.rotation = rotation;
        }
        self
    }

    /// Enables JSON format for logging.
    #[must_use]
    pub const fn json(mut self) -> Self {
        self.config.json = true;
        self
    }
}

impl<F: Sealed> LoggerBuilder<WithName, F> {
    #[must_use]
    pub const fn level(mut self, level: LevelFilter) -> Self {
        self.config.level = level;
        self
    }
    #[must_use]
    pub fn env_filter(mut self, filter: impl Into<String>) -> Self {
        self.config.env_filter = Some(filter.into());
        self
    }
    #[must_use]
    pub const fn console(mut self, enabled: bool) -> Self {
        self.config.console = enabled;
        self
    }

    #[cfg(feature = "opentelemetry")]
    #[must_use]
    pub const fn opentelemetry(mut self, enabled: bool) -> Self {
        self.config.opentelemetry = enabled;
        self
    }

    pub fn path(self, path: impl Into<PathBuf>) -> LoggerBuilder<WithName, WithFile> {
        let mut config = self.config;
        config.path = Some(path.into());
        LoggerBuilder { config, name: self.name, file_state: std::marker::PhantomData }
    }

    /// Initializes the global tracing subscriber and returns a logging system handle.
    ///
    /// ### Architectural Considerations:
    ///
    /// 1. **Non-blocking I/O (Worker Guards)**:
    ///    This method configures background worker threads for log propagation (via `tracing-appender`).
    ///    The returned [`Logger`] struct contains [`WorkerGuard`] instances. **You must keep this
    ///    object in scope** (typically in `main.rs`) for the duration of the program.
    ///    When the `Logger` is dropped, all internal buffers are flushed, and background threads
    ///    are gracefully shut down.
    ///
    /// 2. **Configuration Priority (Cloud-Native)**:
    ///    Builder settings serve as defaults but can be dynamically overridden by environment variables:
    ///    - `LOG_JSON`: Set to `true` or `1` to enable structured JSON output.
    ///    - `LOG_ANSI`: Set to `true` or `1` to enable colored terminal output (text mode only).
    ///    - `LOG_SPANS`: Set to `true` or `1` to include active span lists in JSON logs.
    ///    - `LOG_IGNORE_FIELDS`: Comma-separated list of fields to redact (e.g., `password, token`).
    ///    - `RUST_LOG`: Standard directives for level filtering (e.g., `info,nx_core=debug`).
    ///
    /// 3. **Global Singleton State**:
    ///    This method attempts to set the global default subscriber. In Rust, a global subscriber
    ///    can only be set **once**. Subsequent calls to `init()` will return a [`LoggerError`]
    ///    indicating that the subscriber is already initialized.
    ///
    /// ### Errors
    /// Returns [`LoggerError`] if:
    /// - The provided logger name is empty or contains only whitespace.
    /// - A global subscriber has already been registered.
    /// - (File-mode) Failed to create the log directory or initialize the file appender.
    /// - The `env_filter` string contains invalid syntax.
    ///
    /// ### Example
    /// ```rust
    /// use nx_logger::Logger;
    ///
    /// fn main() {
    ///     // Initialize the system. Guards are stored in the _logger variable.
    ///     let _logger = Logger::builder()
    ///         .name("nx-service")
    ///         .console(true)
    ///         .init()
    ///         .expect("Failed to setup logging");
    ///
    ///     tracing::info!("Telemetry system is active");
    /// } // _logger goes out of scope here: buffers are flushed to disk/stdout.
    /// ```
    #[allow(clippy::redundant_clone)]
    pub fn init(self) -> Result<Logger, LoggerError> {
        validate_config(&self.config, &self.name.0)?;
        let env_filter = build_env_filter(&self.config)?;

        let is_json =
            std::env::var("LOG_JSON").map_or(self.config.json, |v| v == "1" || v == "true");
        let use_ansi = std::env::var("LOG_ANSI").map_or(true, |v| v == "1" || v == "true");
        let include_spans = std::env::var("LOG_SPANS").map_or(true, |v| v == "1" || v == "true");
        let ignored_fields: Vec<String> = std::env::var("LOG_IGNORE_FIELDS")
            .unwrap_or_else(|_| "password,details,backtrace,target,code,kind,context".to_owned())
            .split(',')
            .map(|s| s.trim().to_owned())
            .collect();

        let mut layers = Vec::new();

        #[cfg(not(target_arch = "wasm32"))]
        let mut guards = Vec::new();

        // 1. Profiling
        #[cfg(all(not(target_arch = "wasm32"), feature = "profiling", tokio_unstable))]
        if self.config.console {
            layers.push(console_subscriber::spawn().boxed());
        }

        // 2. OpenTelemetry
        #[cfg(all(not(target_arch = "wasm32"), feature = "opentelemetry-otlp"))]
        let mut otel_guard = None;

        #[cfg(all(not(target_arch = "wasm32"), feature = "opentelemetry-otlp"))]
        if self.config.opentelemetry {
            global::set_text_map_propagator(TraceContextPropagator::new());
            let (guard, log_layer) = init_otlp_pipeline(self.name.0.clone())?;
            otel_guard = Some(guard);

            let tracer = global::tracer(self.name.0.clone());

            let otel_trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            layers.push(otel_trace_layer.boxed());
            layers.push(log_layer.boxed());
        }

        // 3. Console Layer
        if self.config.console {
            #[cfg(not(target_arch = "wasm32"))]
            let (writer, guard) = tracing_appender::non_blocking(std::io::stderr());
            #[cfg(not(target_arch = "wasm32"))]
            guards.push(guard);

            #[cfg(target_arch = "wasm32")]
            let writer = std::io::stderr;

            let layer = if is_json {
                layer().json().with_span_list(include_spans).with_writer(writer).boxed()
            } else {
                layer()
                    .compact()
                    .with_ansi(use_ansi)
                    .fmt_fields(FilteredFields { ignored: ignored_fields.clone() })
                    .boxed()
            };
            layers.push(layer);
        }

        // 4. File Layer
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = self.config.path {
            std::fs::create_dir_all(&path).with_details("Failed to create log dir")?;
            let file_appender = tracing_appender::rolling::Builder::new()
                .rotation(self.config.rotation)
                .filename_prefix(&self.name.0)
                .filename_suffix(LOG_FILE_SUFFIX)
                .max_log_files(self.config.max_files)
                .build(path)
                .map_err(|e| LoggerError::appender().with_details_fn(|| e.to_string()))?;

            let (writer, guard) = tracing_appender::non_blocking(file_appender);
            guards.push(guard);

            let file_layer = layer().with_writer(writer).with_ansi(false);
            let boxed = if is_json {
                file_layer.json().with_span_list(include_spans).boxed()
            } else {
                file_layer.fmt_fields(FilteredFields { ignored: ignored_fields }).boxed()
            };
            layers.push(boxed);
        }

        if layers.is_empty() {
            return Err(LoggerError::invalid_configuration().with_details("No layers enabled"));
        }

        tracing_subscriber::registry().with(env_filter).with(layers).try_init()?;

        Ok(Logger {
            #[cfg(not(target_arch = "wasm32"))]
            guards,
            #[cfg(all(not(target_arch = "wasm32"), feature = "opentelemetry-otlp"))]
            _otel_guard: otel_guard,
        })
    }
}

/// A handle to the initialized logging system. Holds background worker guards.
#[must_use = "Dropping the Logger handle will stop background logging threads."]
#[derive(Debug)]
pub struct Logger {
    #[cfg(not(target_arch = "wasm32"))]
    guards: Vec<WorkerGuard>,
    #[cfg(all(not(target_arch = "wasm32"), feature = "opentelemetry-otlp"))]
    _otel_guard: Option<OpenTelemetryGuard>,
}

impl Logger {
    #[must_use]
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder {
            config: LoggerConfig::default(),
            name: NoName,
            file_state: std::marker::PhantomData,
        }
    }

    pub fn flush(&self) {
        tracing::debug!("Requesting logger flush");
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if !self.guards.is_empty() {
            tracing::info!("Shutting down logging system, flushing buffers...");
        }
    }
}

// --- Field Filtering Logic ---

macro_rules! forward_filtered {
    ($($method:ident($val_ty:ty)),*) => {
        $(
            fn $method(&mut self, field: &Field, value: $val_ty) {
                if !self.ignored.iter().any(|i| i == field.name()) {
                    self.inner.$method(field, value);
                }
            }
        )*
    };
}

#[derive(Debug, Clone)]
struct FilteredFields {
    ignored: Vec<String>,
}

impl<'writer> FormatFields<'writer> for FilteredFields {
    fn format_fields<R>(&self, mut writer: Writer<'writer>, fields: R) -> std::fmt::Result
    where
        R: tracing_subscriber::field::RecordFields,
    {
        let visitor = DefaultFields::new().make_visitor(writer.by_ref());
        let mut filtering_visitor = FilteringVisitor { inner: visitor, ignored: &self.ignored };

        fields.record(&mut filtering_visitor);
        Ok(())
    }
}

struct FilteringVisitor<'a, V> {
    inner: V,
    ignored: &'a [String],
}

impl<V: Visit> Visit for FilteringVisitor<'_, V> {
    forward_filtered!(
        record_str(&str),
        record_f64(f64),
        record_i64(i64),
        record_u64(u64),
        record_i128(i128),
        record_u128(u128),
        record_bool(bool),
        record_debug(&dyn std::fmt::Debug),
        record_error(&(dyn std::error::Error + 'static))
    );
}

// --- Helpers ---

fn validate_config(
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))] config: &LoggerConfig,
    name: &str,
) -> Result<(), LoggerError> {
    if name.trim().is_empty() {
        return Err(
            LoggerError::invalid_configuration().with_details("Logger name cannot be empty")
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    if config.max_files == 0 {
        return Err(LoggerError::invalid_configuration().with_details("`max_files` must be > 0"));
    }
    Ok(())
}

fn build_env_filter(config: &LoggerConfig) -> Result<EnvFilter, LoggerError> {
    let builder = EnvFilter::builder().with_default_directive(config.level.into());
    config.env_filter.as_ref().map_or_else(
        || Ok(builder.from_env_lossy()),
        |filter| {
            builder.parse(filter).map_err(|e| {
                LoggerError::invalid_configuration()
                    .with_details(format!("Invalid filter `{filter}`: {e}"))
            })
        },
    )
}

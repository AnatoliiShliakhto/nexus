/// Errors that can occur during logger initialization or telemetry operation.
#[nx_error::error]
pub enum LoggerError {
    /// Failure during local file operations such as directory creation or log rotation.
    #[error(
        message = "Filesystem I/O error",
        source = std::io::Error,
        code = "LOGGER_IO_ERROR"
    )]
    Io,

    /// Provided configuration parameters are invalid or incompatible.
    #[error(message = "Invalid configuration", code = "LOGGER_CONFIG_ERROR")]
    InvalidConfiguration,

    /// Failure specific to the rolling file appender setup.
    #[error(message = "Failed to initialize log file appender", code = "LOGGER_APPENDER_ERROR")]
    Appender,

    /// Indicates that a global tracing subscriber is already active in the process.
    #[error(
        message = "Tracing subscriber already initialized",
        source = tracing_subscriber::util::TryInitError,
        code = "LOGGER_SUBSCRIBER_ERROR"
    )]
    Subscriber,

    /// Failure while connecting to or configuring the `OpenTelemetry` collector.
    #[cfg(feature = "opentelemetry")]
    #[error(
        message = "Telemetry exporter initialization failed",
        source = opentelemetry_otlp::ExporterBuildError,
        code = "LOGGER_OPENTELEMETRY_ERROR"
    )]
    OpenTelemetry,
}

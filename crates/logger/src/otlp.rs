use crate::error::{LoggerError, LoggerErrorExt};
use opentelemetry::{KeyValue, global};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::{SdkLogger, SdkLoggerProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;

/// A guard that shuts down the global `OpenTelemetry` tracer provider on a drop.
#[derive(Debug)]
pub struct OpenTelemetryGuard {
    tracer_provider: SdkTracerProvider,
    logger_provider: SdkLoggerProvider,
}

impl OpenTelemetryGuard {
    /// Explicitly shuts down the tracer provider.
    pub fn shutdown(self) {
        drop(self);
    }
}

impl Drop for OpenTelemetryGuard {
    fn drop(&mut self) {
        let _ = self.tracer_provider.shutdown();
        let _ = self.logger_provider.shutdown();
    }
}

/// Installs an OTLP tracer provider and sets it as the global tracer and logger providers.
pub(crate) fn init_otlp_pipeline(
    service_name: String,
) -> Result<
    (OpenTelemetryGuard, OpenTelemetryTracingBridge<SdkLoggerProvider, SdkLogger>),
    LoggerError,
> {
    let resource = Resource::builder_empty()
        .with_attributes([KeyValue::new("service.name", service_name)])
        .build();

    let tracer_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .with_details("Failed to build OTLP span exporter")?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(tracer_exporter)
        .with_resource(resource.clone())
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .build()
        .with_details("Failed to build OTLP log exporter")?;

    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(resource)
        .build();

    let layer = OpenTelemetryTracingBridge::new(&logger_provider);

    Ok((OpenTelemetryGuard { tracer_provider, logger_provider }, layer))
}

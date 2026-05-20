use crate::error::LoggerError;
use crate::utils::{FieldValue, FieldVisitor, IgnoreFields, SpanAttributes};
use opentelemetry::logs::{AnyValue, LogRecord, Logger, LoggerProvider, Severity};
use opentelemetry::{Key, KeyValue, global};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::{SdkLogger, SdkLoggerProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use smallvec::SmallVec;
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

const MAX_SPAN_ATTRIBUTES: usize = 16;

#[derive(Debug)]
pub(crate) struct OpenTelemetryGuard {
    tracer_provider: SdkTracerProvider,
    logger_provider: SdkLoggerProvider,
}
impl Drop for OpenTelemetryGuard {
    fn drop(&mut self) {
        let _ = self.tracer_provider.shutdown();
        let _ = self.logger_provider.shutdown();
    }
}

pub(crate) struct FilteredOtlpLayer {
    logger: SdkLogger,
    ignored: IgnoreFields,
}

impl FilteredOtlpLayer {
    fn new(provider: &SdkLoggerProvider, name: &str, ignored: IgnoreFields) -> Self {
        Self { logger: provider.logger(name.to_owned()), ignored }
    }
}

impl<S> Layer<S> for FilteredOtlpLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let meta = event.metadata();
        let mut attrs: SmallVec<(Key, AnyValue), 16> = SmallVec::new();
        attrs.push((Key::new("target"), AnyValue::from(meta.target())));

        let mut fields: SmallVec<(&'static str, FieldValue), 16> = SmallVec::new();
        event.record(&mut FieldVisitor { attrs: &mut fields, ignored: &self.ignored });

        let mut message = None;
        for (k, v) in fields {
            if k == "message" {
                message = Some(v);
                continue;
            }
            attrs.push((Key::new(k), field_to_any(v)));
        }

        if let Some(scope) = ctx.event_scope(event) {
            let mut attr_count = 0;

            for span in scope.from_root() {
                if attr_count >= MAX_SPAN_ATTRIBUTES {
                    break;
                }

                attrs.push((Key::new("span.name"), AnyValue::from(span.name().to_owned())));
                attr_count += 1;

                if let Some(a) = span.extensions().get::<SpanAttributes>() {
                    for (k, v) in &a.attrs {
                        if attr_count >= MAX_SPAN_ATTRIBUTES {
                            break;
                        }
                        attrs.push((Key::new(*k), field_to_any(v.clone())));
                        attr_count += 1;
                    }
                }
            }
        }

        let mut rec = self.logger.create_log_record();
        rec.set_severity_number(match *meta.level() {
            tracing::Level::TRACE => Severity::Trace,
            tracing::Level::DEBUG => Severity::Debug,
            tracing::Level::INFO => Severity::Info,
            tracing::Level::WARN => Severity::Warn,
            tracing::Level::ERROR => Severity::Error,
        });
        rec.add_attributes(attrs);

        let body = match message {
            Some(FieldValue::Static(s)) => AnyValue::from(s),
            Some(FieldValue::String(s)) => AnyValue::from(s.to_string()),
            Some(v) => AnyValue::from(v.to_string()),
            None => AnyValue::from(meta.target()),
        };
        rec.set_body(body);

        self.logger.emit(rec);
    }
}

#[inline]
fn field_to_any(v: FieldValue) -> AnyValue {
    match v {
        FieldValue::Static(s) => AnyValue::from(s),
        FieldValue::String(s) => AnyValue::from(s.to_string()),
        FieldValue::I64(i) => AnyValue::from(i),
        FieldValue::U64(u) => AnyValue::from(u.cast_signed()),
        FieldValue::F64(f) => AnyValue::from(f),
        FieldValue::Bool(b) => AnyValue::from(b),
    }
}

pub(crate) fn init_otlp_pipeline(
    service_name: &str,
    ignored: IgnoreFields,
) -> Result<(OpenTelemetryGuard, FilteredOtlpLayer), LoggerError> {
    let res = Resource::builder_empty()
        .with_attributes([KeyValue::new("service.name", service_name.to_owned())])
        .build();

    let tp = SdkTracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .build()
                .map_err(|e| LoggerError::invalid_configuration().with_details(e.to_string()))?,
        )
        .with_resource(res.clone())
        .build();

    global::set_tracer_provider(tp.clone());

    let lp = SdkLoggerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::LogExporter::builder()
                .with_tonic()
                .build()
                .map_err(|e| LoggerError::invalid_configuration().with_details(e.to_string()))?,
        )
        .with_resource(res)
        .build();

    Ok((
        OpenTelemetryGuard { tracer_provider: tp, logger_provider: lp.clone() },
        FilteredOtlpLayer::new(&lp, service_name, ignored),
    ))
}

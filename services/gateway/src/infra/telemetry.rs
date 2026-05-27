use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::trace::{TraceContextExt, TraceId};
use opentelemetry::{KeyValue, global};
use std::sync::LazyLock;
use std::time::Duration;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultOnRequest, TraceLayer};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub(crate) static METRICS: LazyLock<Metrics> = LazyLock::new(Metrics::new);

pub(crate) fn tracing_layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    impl Fn(&Request<Body>) -> tracing::Span + Clone,
    DefaultOnRequest,
    impl Fn(&Response<Body>, Duration, &tracing::Span) + Clone,
> {
    TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Body>| {
            let parent_context = global::get_text_map_propagator(|propagator| {
                propagator.extract(&opentelemetry_http::HeaderExtractor(request.headers()))
            });

            let span = tracing::info_span!(
                "http_request",
                "service" = env!("CARGO_PKG_NAME"),
                "http.method" = %request.method(),
                "http.route" = %request.uri().path(),
                "http.status_code" = tracing::field::Empty,
                "traceid" = tracing::field::Empty
            );

            let _ = span.set_parent(parent_context);

            span
        })
        .on_response(|response: &Response<Body>, latency: Duration, span: &tracing::Span| {
            let status = response.status();
            let trace_id = span.context().span().span_context().trace_id();

            span.record("http.status_code", status.as_u16());
            if trace_id != TraceId::INVALID {
                span.record("traceid", trace_id.to_string());
            }

            METRICS.record_request(status, latency)
        })
}

pub(crate) async fn set_trace_id_header(request: Request<Body>, next: Next) -> impl IntoResponse {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    if headers.contains_key("x-trace-id") {
        return response;
    }

    let trace_id = tracing::Span::current().context().span().span_context().trace_id();

    if trace_id != TraceId::INVALID {
        let mut buf = [0u8; 32];
        if hex::encode_to_slice(trace_id.to_bytes(), &mut buf).is_ok() {
            if let Ok(header_value) = HeaderValue::from_bytes(&buf) {
                headers.insert("x-trace-id", header_value);
            }
        }
    }

    response
}

// --- Metrics ---

#[derive(Debug)]
pub(crate) struct Metrics {
    requests_total: Counter<u64>,
    request_duration: Histogram<f64>,

    db_errors_total: Counter<u64>,
    db_query_duration: Histogram<f64>,

    proxy_requests_total: Counter<u64>,
    proxy_request_duration: Histogram<f64>,
    proxy_errors_total: Counter<u64>,
}

impl Metrics {
    fn new() -> Self {
        let meter = global::meter(env!("CARGO_PKG_NAME"));

        Self {
            requests_total: meter
                .u64_counter("nx_gateway_http_requests_total")
                .with_unit("{count}")
                .with_description("Total number of incoming HTTP requests to the gateway")
                .build(),
            request_duration: meter
                .f64_histogram("nx_gateway_http_request_duration_seconds")
                .with_unit("s")
                .with_description("Gateway incoming HTTP request latency in seconds")
                .build(),

            db_errors_total: meter
                .u64_counter("nx_gateway_db_errors_total")
                .with_unit("{error}")
                .with_description("Total number of database connection or statement errors")
                .build(),
            db_query_duration: meter
                .f64_histogram("nx_gateway_db_query_duration_seconds")
                .with_unit("s")
                .with_description("Database query execution latency in seconds")
                .build(),

            proxy_requests_total: meter
                .u64_counter("nx_gateway_proxy_requests_total")
                .with_unit("{count}")
                .with_description("Total number of proxied HTTP requests to upstream components")
                .build(),
            proxy_request_duration: meter
                .f64_histogram("nx_gateway_proxy_request_duration_seconds")
                .with_unit("s")
                .with_description("Proxy request latency to upstream components in seconds")
                .build(),
            proxy_errors_total: meter
                .u64_counter("nx_gateway_proxy_errors_total")
                .with_unit("{error}")
                .with_description("Total number of proxy dispatch or upstream connection errors")
                .build(),
        }
    }

    pub(crate) fn record_request(&self, status: StatusCode, latency: Duration) {
        let labels = [KeyValue::new("status", status_to_str(status))];
        self.requests_total.add(1, &labels);
        self.request_duration.record(latency.as_secs_f64(), &labels);
    }

    pub(crate) fn record_db_error(&self, error_type: &'static str) {
        let labels = [KeyValue::new("type", error_type)];
        self.db_errors_total.add(1, &labels);
    }

    pub(crate) fn record_db_query_duration(&self, duration_secs: f64) {
        self.db_query_duration.record(duration_secs, &[]);
    }

    pub(crate) fn record_proxy_request(&self, status: StatusCode, duration: f64) {
        let labels = [KeyValue::new("status", status_to_str(status))];
        self.proxy_requests_total.add(1, &labels);
        self.proxy_request_duration.record(duration, &labels);
    }

    pub(crate) fn record_proxy_error(&self, error_type: &'static str) {
        let labels = [KeyValue::new("error", error_type)];
        self.proxy_errors_total.add(1, &labels);
    }
}

// --- Helpers ---

/// Maps StatusCode to a `&'static str` for `OpenTelemetry` labels.
/// Necessary because OTEL labels require static lifetimes to avoid allocations.
fn status_to_str(status: StatusCode) -> &'static str {
    match status.as_u16() {
        200 => "200",
        201 => "201",
        204 => "204",
        301 => "301",
        302 => "302",
        304 => "304",
        400 => "400",
        401 => "401",
        403 => "403",
        404 => "404",
        409 => "409",
        422 => "422",
        429 => "429",
        500 => "500",
        502 => "502",
        503 => "503",
        504 => "504",
        _ => "other",
    }
}

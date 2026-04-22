use axum::body::Body;
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::IntoResponse;
use opentelemetry::global;
use opentelemetry::trace::TraceContextExt;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::TraceLayer;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub(crate) fn tracing_layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    impl Fn(&Request<Body>) -> tracing::Span + Clone,
> {
    TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
        let parent_context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&opentelemetry_http::HeaderExtractor(request.headers()))
        });

        let span = tracing::info_span!(
            "http_request",
            "service" = env!("CARGO_PKG_NAME"),
            "http.method" = %request.method(),
            "http.uri" = %request.uri(),
            "trace_id" = tracing::field::Empty
        );

        let _ = span.set_parent(parent_context);
        let trace_id = span.context().span().span_context().trace_id().to_string();
        span.record("trace_id", trace_id);

        span
    })
}

pub(crate) async fn set_trace_id_header(request: Request<Body>, next: Next) -> impl IntoResponse {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    if headers.contains_key("x-trace-id") {
        return response;
    }

    let trace_id = tracing::Span::current().context().span().span_context().trace_id().to_string();

    if let Ok(header_value) = HeaderValue::from_str(&trace_id) {
        headers.insert("x-trace-id", header_value);
    }

    response
}

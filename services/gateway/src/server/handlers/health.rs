use axum::Json;
use axum::http::header;
use axum::response::IntoResponse;
use serde::Serialize;
use std::sync::LazyLock;
use tokio::time::Instant;

/// Health check response
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct HealthResponse {
    /// Status
    status: &'static str,
    /// Version
    version: &'static str,
    /// Uptime in seconds
    uptime: u64,
}

pub(crate) static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

pub(crate) async fn health_handler() -> impl IntoResponse {
    let body = HealthResponse {
        status: "up",
        version: env!("CARGO_PKG_VERSION"),
        uptime: START_TIME.elapsed().as_secs(),
    };

    (
        [
            (header::CACHE_CONTROL, "no-store, no-cache, must-revalidate"),
            (header::PRAGMA, "no-cache"),
        ],
        Json(body),
    )
}

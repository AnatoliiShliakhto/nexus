use super::handlers;
use super::state::GatewayState;
use crate::core::config::GatewayConfig;
use crate::core::error::handle_tower_error;
use crate::error::GatewayError;
use crate::features::proxy::middleware::ProxyFilterLayer;
use crate::infra::telemetry::{set_trace_id_header, tracing_layer};
use axum::error_handling::HandleErrorLayer;
use axum::routing::{get, post};
use axum::{Router, middleware};
use http::{HeaderName, HeaderValue, Method, header};
use std::time::Duration;
use tower::ServiceBuilder;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::load_shed::LoadShedLayer;
use tower_http::cors::CorsLayer;

pub(crate) async fn app(config: GatewayConfig) -> Result<Router, GatewayError> {
    let settings = config.get();

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static("dpop"),
        ])
        .allow_credentials(true);

    let protection_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_tower_error))
        .layer(BufferLayer::new(settings.traffic.buffer_size))
        .layer(RateLimitLayer::new(
            settings.traffic.rate_limit_requests,
            Duration::from_secs(settings.traffic.rate_limit_duration_secs),
        ))
        .layer(LoadShedLayer::new())
        .concurrency_limit(settings.traffic.concurrency_limit);

    let telemetry_stack = ServiceBuilder::new()
        .layer(tracing_layer())
        .layer(middleware::from_fn(set_trace_id_header));

    let state = GatewayState::init(config).await?;

    let proxy_layer = ProxyFilterLayer::new(state);

    let router = Router::new()
        .layer(proxy_layer)
        .route(
            "/api/auth/tokens",
            post(handlers::auth::authorization_handler)
                .delete(handlers::auth::token_revoke_handler),
        )
        .route("/api/auth/tokens/refresh", post(handlers::auth::token_refresh_handler))
        .route("/api/health", get(handlers::health::health_handler))
        .layer(cors)
        .layer(protection_stack)
        .with_state(state)
        .layer(telemetry_stack);

    Ok(router)
}

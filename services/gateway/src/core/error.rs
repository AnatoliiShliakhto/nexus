use axum::extract::rejection::JsonRejection;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use nx_error::prelude::*;
use tower::BoxError;

/// Represents all possible errors that can occur within the Nexus API Gateway.
///
/// Error codes follow the `GW_[CATEGORY]_[SPECIFIC_ERROR]` format for easier
/// log aggregation and observability tracing.
#[error]
pub enum GatewayError {
    #[transparent(crate::infra::database::DatabaseError)]
    Database,

    #[transparent(crate::features::identity::error::IdentityError)]
    Identity,

    // ------------------------------------------------------------------------
    // Infrastructure & Storage Errors
    // ------------------------------------------------------------------------
    /// Triggered when the Redis client fails to connect or execute a command.
    /// This is vital for distributed features like DPoP replay protection or rate limiting.
    #[error(
        message = "An infrastructure error occurred",
        status = ErrorStatus::InternalServerError,
        code = "GW_INFRA_REDIS_ERROR",
        source = redis::RedisError,
    )]
    Redis,

    // ------------------------------------------------------------------------
    // Configuration & Lifecycle Errors
    // ------------------------------------------------------------------------
    /// Transparently wraps configuration loading or parsing errors.
    #[transparent(nx_config::error::ConfigError)]
    Config,

    /// Triggered when the gateway fails to hook into OS signals (e.g., SIGTERM, SIGINT).
    #[error(
        message = "Failed to initialize shutdown signal handlers",
        status = ErrorStatus::InternalServerError,
        code = "GW_SYS_SHUTDOWN_INIT_FAILED",
    )]
    SignalInitFailed,

    // ------------------------------------------------------------------------
    // Traffic & Load Management Errors
    // ------------------------------------------------------------------------
    /// Triggered when the upstream service fails to respond within the allowed time.
    /// Changed to `GatewayTimeout` (504) as this is the standard for proxy timeouts.
    #[error(
        message = "Upstream service failed to respond in time",
        status = ErrorStatus::GatewayTimeout,
        code = "GW_TRAFFIC_UPSTREAM_TIMEOUT",
    )]
    GatewayTimeout,

    /// Triggered by the concurrency or rate limiter when the gateway is under a heavy load.
    #[error(
        message = "Gateway is temporarily overloaded due to high traffic",
        status = ErrorStatus::ServiceUnavailable,
        code = "GW_TRAFFIC_OVERLOADED",
    )]
    ServiceOverloaded,

    // ------------------------------------------------------------------------
    // Routing & Proxy Errors
    // ------------------------------------------------------------------------
    /// Triggered when a matched route points to an unparsable or malformed target URI.
    /// Changed from `BadRequest` (400) to `BadGateway` (502) because this is a server-side routing issue.
    #[error(
        message = "The resolved upstream target URI is invalid",
        status = ErrorStatus::BadGateway,
        code = "GW_PROXY_TARGET_INVALID",
        source = url::ParseError,
    )]
    InvalidTargetUri,

    /// Triggered when the gateway fails to parse parts of the incoming or outgoing URI.
    #[error(
        message = "Failed to parse URI components during routing",
        status = ErrorStatus::InternalServerError,
        code = "GW_PROXY_URI_PARSE_FAILED",
    )]
    UriParseFailed,

    /// Triggered when a request matches a route, but the target URL is completely missing.
    #[error(
        message = "Upstream target URL is missing for the matched route",
        status = ErrorStatus::BadGateway,
        code = "GW_PROXY_TARGET_MISSING",
    )]
    MissingTargetUrl,

    /// Triggered when the Hyper client fails to construct the proxy HTTP request.
    #[error(
        message = "Failed to build the upstream proxy request",
        status = ErrorStatus::InternalServerError,
        code = "GW_PROXY_REQUEST_BUILD_FAILED",
    )]
    RequestBuildFailed,

    /// Triggered when the gateway fails to dispatch the request to the upstream service.
    /// This usually indicates network issues, connection resets, or DNS failures.
    #[error(
        message = "Failed to dispatch request to upstream service",
        status = ErrorStatus::BadGateway,
        code = "GW_PROXY_DISPATCH_FAILED",
    )]
    DispatchFailed,

    #[error(
        message = "You do not have permission to access this resource",
        status = ErrorStatus::Forbidden,
        code = "GW_AUTH_ACCESS_DENIED",
    )]
    AccessDenied,

    #[error(
        message = "Invalid request payload",
        status = ErrorStatus::UnprocessableEntity,
        code = "GW_PAYLOAD_INVALID_FORMAT",
    )]
    InvalidPayload,

    // ------------------------------------------------------------------------
    // Generic Errors
    // ------------------------------------------------------------------------
    /// A catch-all for unexpected internal gateway panics or unhandled logic states.
    #[error(
        message = "An unexpected internal gateway error occurred",
        status = ErrorStatus::InternalServerError,
        code = "GW_SYS_INTERNAL_ERROR",
    )]
    Internal,
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let status = self.status().as_u16();

        let body = if cfg!(debug_assertions) {
            self.to_detailed_json_string()
        } else {
            self.to_json_string()
        };

        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-content-type-options", "nosniff")
            .body(axum::body::Body::from(body))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
}

pub(crate) async fn handle_tower_error(err: BoxError) -> impl IntoResponse {
    if err.is::<tower::timeout::error::Elapsed>() {
        GatewayError::gateway_timeout()
    } else if err.is::<tower::load_shed::error::Overloaded>() {
        GatewayError::service_overloaded()
    } else {
        GatewayError::internal().with_details(err.to_string())
    }
}

impl GatewayError {
    pub(crate) fn emit(&self) {
        tracing::error!(
            status = self.status().as_u16(),
            code = %self.code(),
            details = %self.details().as_deref().unwrap_or(""),
            "{}", self.message(),
        );
    }
}

impl From<JsonRejection> for GatewayError {
    fn from(rejection: JsonRejection) -> Self {
        let error = GatewayError::invalid_payload().with_message(rejection.body_text());
        error
    }
}

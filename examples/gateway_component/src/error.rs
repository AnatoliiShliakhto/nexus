use nx_http::error::prelude::*;

#[error]
pub(crate) enum GatewayError {
    #[transparent(
        source = nx_http::error::Error,
        from = [
            nx_http::spin_tools::environment::SpinEnvironmentError,
            nx_http::spin_tools::proxy::ProxyRequestError,
            nx_http::request::RequestError,
            nx_http::spin_sdk::wasip3::http::types::ErrorCode,
            nx_http::url::ParseError,
        ],
    )]
    Http,

    #[error(
        message = "Resource not found",
        status = ErrorStatus::NotFound,
        code = "GATEWAY_RESORCE_NOT_FOUND",
    )]
    NotFound,

    #[error(
        message = "HTTP method is not supported for this route",
        status = ErrorStatus::MethodNotAllowed,
        code = "GATEWAY_METHOD_NOT_ALLOWED"
    )]
    MethodNotAllowed,

    #[error(
        message = "Route is misconfigured: missing upstream target",
        status = ErrorStatus::InternalServerError,
        code = "GATEWAY_ROUTE_MISCONFIGURED"
    )]
    TargetMissing,

    #[error(
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "GATEWAY_INTERNAL_ERROR"
    )]
    Internal,
}

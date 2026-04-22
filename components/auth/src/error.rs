use nx_http::error::prelude::*;

#[error]
pub(crate) enum AuthError {
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
        code = "AUTH_RESOURCE_NOT_FOUND",
    )]
    NotFound,

    #[error(
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "AUTH_INTERNAL_ERROR",
    )]
    Internal,
}

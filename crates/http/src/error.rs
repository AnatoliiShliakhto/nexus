pub use nx_error::*;
#[error]
pub enum Error {
    #[cfg(feature = "spin")]
    #[transparent(crate::spin_tools::proxy::ProxyRequestError)]
    Proxy,

    #[cfg(feature = "spin")]
    #[transparent(crate::spin_tools::environment::SpinEnvironmentError)]
    Environment,

    #[error(
        message = "HTTP request build failed",
        status = ErrorStatus::InternalServerError,
        code = "HTTP_REQUEST_BUILD_FAILURE",
        source = crate::request::RequestError,
    )]
    Request,

    #[cfg(feature = "spin")]
    #[error(
        message = "Upstream service is unreachable or failed to respond",
        status = ErrorStatus::ServiceUnavailable,
        code = "UPSTREAM_UNAVAILABLE",
        source = spin_sdk::wasip3::http::types::ErrorCode,
    )]
    Unreachable,

    #[error(
        message = "Invalid URL format",
        status = ErrorStatus::BadRequest,
        code = "URL_INVALID",
        source = url::ParseError
    )]
    UrlInvalid,
}

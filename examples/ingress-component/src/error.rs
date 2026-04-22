use nx_http::error::prelude::*;

#[error]
pub(crate) enum IngressError {
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
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "INGRESS_INTERNAL_ERROR",
    )]
    Internal,
}

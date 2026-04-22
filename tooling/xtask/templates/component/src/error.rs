use nx_http::error::prelude::*;

#[error]
pub(crate) enum {{ shortname | pascal_case }}Error {
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
        code = "{{ shortname | shouty_snake_case }}_INTERNAL_ERROR",
    )]
    Internal,
}

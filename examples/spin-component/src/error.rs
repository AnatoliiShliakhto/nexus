use nx_http::error::prelude::*;

#[error]
pub(crate) enum SpinComponentError {
    /// Configuration errors (e.g., missing or invalid Spin variables).
    #[transparent(source = nx_http::spin_tools::environment::SpinVariableError)]
    Env,

    /// Errors occurring during the construction of a proxy request.
    #[transparent(source = nx_http::spin_tools::proxy::ProxyRequestError)]
    Proxy,

    /// Failures when communicating with upstream services (WASI-HTTP outbound).
    #[error(
        message = "Upstream service is unreachable or failed to respond",
        status = ErrorStatus::ServiceUnavailable,
        code = "SPIN_COMPONENT_UPSTREAM_UNAVAILABLE",
        source = nx_http::spin_sdk::http::SendError,
    )]
    Upstream,

    /// Catch-all for unexpected internal processing failures.
    #[error(
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "SPIN_COMPONENT_INTERNAL_ERROR",
    )]
    Internal,
}

use nx_http::error::prelude::*;

#[error]
pub(crate) enum WasiComponentError {
    /// Catch-all for unexpected internal processing failures.
    #[error(
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "WASI_COMPONENT_INTERNAL_ERROR",
    )]
    Internal,
}

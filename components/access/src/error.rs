use nx_http::error::prelude::*;

#[error]
pub(crate) enum AccessError {
    #[error(
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "ACCESS_INTERNAL_ERROR",
    )]
    Internal,
}

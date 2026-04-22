use nx_http::error::prelude::*;

#[error]
pub(crate) enum AccountError {
    #[error(
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "ACCOUNT_INTERNAL_ERROR",
    )]
    Internal,
}

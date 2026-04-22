use nx_http::error::prelude::*;

#[error]
pub(crate) enum OrganizationError {
    #[error(
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "ORG_INTERNAL_ERROR",
    )]
    Internal,
}

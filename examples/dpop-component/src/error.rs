use nx_http::error::prelude::*;

#[error]
pub(crate) enum DpopError {
    #[transparent(
        source = nx_http::error::Error,
        from = [
            nx_http::spin_tools::environment::SpinEnvironmentError,
            nx_http::url::ParseError,
        ],
    )]
    Http,

    #[error(
        message = "Invalid DPoP proof format",
        status = ErrorStatus::Unauthorized,
        code = "DPOP_PROOF_INVALID",
    )]
    ProofInvalid,

    #[error(
        message = "DPoP proof signature or claims verification failed",
        status = ErrorStatus::Unauthorized,
        code = "DPOP_VERIFICATION_FAILED",
    )]
    VerificationFailed,

    #[error(
        message = "The provided access token is invalid",
        status = ErrorStatus::Unauthorized,
        code = "AUTH_TOKEN_INVALID",
    )]
    InvalidToken,

    #[error(
        message = "The session has expired",
        status = ErrorStatus::Unauthorized,
        code = "AUTH_TOKEN_EXPIRED",
    )]
    TokenExpired,

    #[error(
        message = "Authorization header is missing",
        status = ErrorStatus::Unauthorized,
        code = "AUTH_TOKEN_MISSING",
    )]
    MissingAuth,

    #[error(
        message = "Failed to deserialize binary payload",
        status = ErrorStatus::BadRequest,
        code = "DPOP_PAYLOAD_INVALID",
        source = postcard::Error,
    )]
    PostCard,

    #[error(
        message = "HTTP method not allowed",
        status = ErrorStatus::MethodNotAllowed,
        code = "DPOP_METHOD_NOT_ALLOWED",
    )]
    MethodNotAllowed,

    #[error(
        message = "Resource not found",
        status = ErrorStatus::NotFound,
        code = "DPOP_RESOURCE_NOT_FOUND",
    )]
    NotFound,

    #[error(
        message = "Invalid request parameters",
        status = ErrorStatus::BadRequest,
        code = "DPOP_BAD_REQUEST",
    )]
    BadRequest,

    #[error(
        message = "Unsupported Content-Type",
        status = ErrorStatus::UnsupportedMediaType,
        code = "DPOP_UNSUPPORTED_CONTENT_TYPE"
    )]
    UnsupportedContent,

    #[error(
        message = "Internal data consistency error",
        status = ErrorStatus::InternalServerError,
        code = "DPOP_INTERNAL_ERROR",
        source = nx_http::spin_sdk::redis::Error,
    )]
    Redis,

    #[error(
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "DPOP_INTERNAL_ERROR",
    )]
    Internal,
}

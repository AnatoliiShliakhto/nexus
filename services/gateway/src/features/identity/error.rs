use nx_error::prelude::*;

#[error]
pub(crate) enum IdentityError {
    #[transparent(crate::infra::database::DatabaseError)]
    Database,

    #[error(
        message = "Invalid login or password",
        status = ErrorStatus::Unauthorized,
        code = "IDENTITY_INVALID_CREDENTIALS",
    )]
    InvalidCredentials,

    #[error(
        message = "Account is disabled or blocked",
        status = ErrorStatus::Forbidden,
        code = "IDENTITY_ACCOUNT_BLOCKED",
    )]
    AccountBlocked,

    #[error(
        message = "Access from this IP address is restricted",
        status = ErrorStatus::Forbidden,
        code = "IDENTITY_IP_RESTRICTED",
    )]
    IpRestricted,

    #[error(
        message = "Authorization header is missing",
        status = ErrorStatus::Unauthorized,
        code = "IDENTITY_AUTHORIZATION_MISSING",
    )]
    AuthorizationMissing,

    #[error(
        message = "Refresh token cookie is missing",
        status = ErrorStatus::Unauthorized,
        code = "IDENTITY_REFRESH_TOKEN_MISSING",
    )]
    RefreshTokenMissing,

    #[error(
        message = "Missing DPoP header",
        status = ErrorStatus::Unauthorized,
        code = "IDENTITY_DPOP_MISSING",
    )]
    HeaderMissing,

    #[error(
        message = "Invalid DPoP proof format",
        status = ErrorStatus::Unauthorized,
        code = "IDENTITY_DPOP_INVALID",
    )]
    ProofInvalid,

    #[error(
        message = "DPoP proof signature or claims verification failed",
        status = ErrorStatus::Unauthorized,
        code = "IDENTITY_DPOP_VERIFICATION_FAILED",
    )]
    VerificationFailed,

    #[error(
        message = "DPoP proof has expired or is outside the accepted time window",
        status = ErrorStatus::Unauthorized,
        code = "IDENTITY_DPOP_EXPIRED",
    )]
    TimeWindowExceeded,

    #[error(
        message = "DPoP proof replayed",
        status = ErrorStatus::Unauthorized,
        code = "IDENTITY_DPOP_REPLAY_DETECTED",
    )]
    ReplayDetected,

    #[error(
        message = "Access token hash (ath) mismatch",
        status = ErrorStatus::Unauthorized,
        code = "IDENTITY_DPOP_ATH_MISMATCH",
    )]
    InvalidTokenHash,

    #[error(
        message = "Unsupported JWK key type or algorithm",
        status = ErrorStatus::BadRequest,
        code = "IDENTITY_DPOP_UNSUPPORTED_KEY",
    )]
    UnsupportedKeyType,

    #[error(
        message = "A new nonce is required to proceed",
        status = ErrorStatus::BadRequest,
        code = "IDENTITY_DPOP_NONCE_REQUIRED",
    )]
    NonceRequired,

    #[error(
        message = "Failed to process session data",
        status = ErrorStatus::InternalServerError,
        code = "IDENTITY_SESSION_DATA_CORRUPTION",
        source = postcard::Error,
    )]
    SessionSerialization,

    #[error(
        message = "Session is invalid, revoked or expired",
        status = ErrorStatus::Unauthorized,
        code = "IDENTITY_SESSION_UNAUTHORIZED",
    )]
    Unauthorized,

    #[error(
        message = "An unexpected internal gateway error occurred",
        status = ErrorStatus::InternalServerError,
        code = "IDENTITY_INTERNAL_ERROR",
    )]
    Internal,
}

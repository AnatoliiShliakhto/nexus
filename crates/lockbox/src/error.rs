use nx_error::prelude::*;

#[error]
pub enum LockboxError {
    #[error(
        message = "Keyring core operation failed",
        status = ErrorStatus::InternalServerError,
        code = "LOCKBOX_CORE_ERROR",
        source = keyring_core::Error,
    )]
    Core,

    #[error(
        message = "No default credential store is configured",
        status = ErrorStatus::NotFound,
        code = "LOCKBOX_NO_STORE",
        help = "Call Lockbox::new() or use_native_store() first.",
    )]
    NoStore,

    #[error(
        message = "The requested store is not supported on this platform",
        status = ErrorStatus::BadRequest,
        code = "LOCKBOX_PLATFORM_NOT_SUPPORTED",
    )]
    NotSupported,

    #[error(
        message = "Unknown or invalid store name provided",
        status = ErrorStatus::NotFound,
        code = "LOCKBOX_INVALID_STORE"
    )]
    InvalidStore,

    #[error(
        message = "JSON serialization/deserialization failed",
        status = ErrorStatus::InternalServerError,
        code = "LOCKBOX_SERIALIZATION_ERROR",
        source = serde_json::Error,
    )]
    Serialization,

    #[error(
        message = "An internal error occurred while managing the lockbox",
        status = ErrorStatus::InternalServerError,
        code = "LOCKBOX_INTERNAL_ERROR"
    )]
    Internal,
}

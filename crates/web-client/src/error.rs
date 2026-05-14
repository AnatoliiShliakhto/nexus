use nx_error::prelude::*;
use serde::Deserialize;
use std::borrow::Cow;

#[error]
pub enum WebClientError {
    #[transparent(nx_lockbox::error::LockboxError)]
    Lockbox,

    #[error(
        message = "API returned an error",
        status = ErrorStatus::InternalServerError,
        code = "WEB_CLIENT_API_ERROR",
    )]
    ApiError,

    #[error(
        message = "Network request failed",
        status = ErrorStatus::InternalServerError,
        code = "WEB_CLIENT_REQUEST_FAILED",
    )]
    RequestFailed,

    #[error(
        message = "Failed to sign the DPoP proof",
        status = ErrorStatus::InternalServerError,
        code = "WEB_CLIENT_DPOP_SIGNING_FAILED",
        source = jsonwebtoken::errors::Error
    )]
    DpopSigningFailed,

    #[error(
        message = "Failed to construct valid HTTP headers",
        status = ErrorStatus::InternalServerError,
        code = "WEB_CLIENT_HEADER_ERROR",
        source = reqwest::header::InvalidHeaderValue
    )]
    InvalidHeader,

    #[error(
        message = "Failed to process JSON data",
        status = ErrorStatus::InternalServerError,
        code = "WEB_CLIENT_SERIALIZATION_ERROR",
        source = serde_json::Error
    )]
    SerializationFailed,

    #[error(
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "WEB_CLIENT_INTERNAL_ERROR"
    )]
    Internal,
}

impl From<reqwest::Error> for WebClientError {
    #[inline]
    fn from(source: reqwest::Error) -> Self {
        let status = source
            .status()
            .map_or(ErrorStatus::InternalServerError, |s| ErrorStatus::from(s.as_u16()));
        let code = Cow::Borrowed(if source.is_timeout() {
            "WEB_CLIENT_REQUEST_TIMEOUT"
        } else if source.is_connect() {
            "WEB_CLIENT_CONNECTION_FAILED"
        } else if source.is_decode() {
            "WEB_CLIENT_DECODE_ERROR"
        } else {
            "WEB_CLIENT_REQUEST_FAILED"
        });
        let adapter =
            WebClientErrorAdapter::<_, __WebClientErrorDpopSigningFailedMarker>::new(source);
        let message = ErrorMetadata::message(&adapter);
        let help = ErrorMetadata::help(&adapter);
        let details = ErrorMetadata::details(&adapter);
        Self::RequestFailed {
            data: Box::new(WebClientErrorData { status, code, message, details, help }),
        }
    }
}

// --- API Error ---
#[derive(Debug, Deserialize)]
pub struct ApiErrorBody {
    pub status: u16,
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

use nx_http::error::prelude::*;
use serde::Deserialize;
use std::borrow::Cow;

#[error]
pub enum DatabaseError {
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
        message = "Unsupported Content-Type",
        status = ErrorStatus::UnsupportedMediaType,
        code = "DATABASE_UNSUPPORTED_CONTENT_TYPE",
    )]
    UnsupportedContent,

    #[error(
        message = "Invalid data format or constraints",
        status = ErrorStatus::UnprocessableEntity,
        code = "DATABASE_DATA_INVALID",
        source = serde_json::Error
    )]
    Json,

    #[error(
        message = "Invalid data format or constraints",
        status = ErrorStatus::UnprocessableEntity,
        code = "DATABASE_DATA_INVALID",
    )]
    FlatBuffers,

    #[error(
        message = "Invalid data format or constraints",
        status = ErrorStatus::UnprocessableEntity,
        code = "DATABASE_DATA_INVALID",
        source = surrealdb_types::Error,
    )]
    SurrealValue,

    #[error(
        message = "Invalid configuration parameters",
        status = ErrorStatus::InternalServerError,
        code = "DATABASE_CONFIG_INVALID"
    )]
    ConfigInvalid,

    #[error(
        message = "Database authentication failed",
        status = ErrorStatus::InternalServerError,
        code = "DATABASE_AUTH_ERROR"
    )]
    Authentication,

    #[error(
        message = "Query execution failed",
        status = ErrorStatus::InternalServerError,
        code = "DATABASE_QUERY_EXECUTION_FAILED"
    )]
    Query,

    #[error(
        message = "Resource not found",
        status = ErrorStatus::NotFound,
        code = "DATABASE_RESOURCE_NOT_FOUND"
    )]
    NotFound,

    #[error(
        message = "An unexpected internal error occurred",
        status = ErrorStatus::InternalServerError,
        code = "DATABASE_INTERNAL_ERROR"
    )]
    Internal,
}

#[derive(Debug, Deserialize)]
struct SurrealError {
    code: u16,
    details: String,
    description: Option<String>,
    information: Option<String>,
}

impl From<&[u8]> for DatabaseError {
    fn from(err_body: &[u8]) -> Self {
        if err_body.is_empty() {
            return Self::internal().with_details("Empty response body");
        }

        let dto = match serde_json::from_slice::<SurrealError>(err_body) {
            Ok(data) => data,
            Err(e) => {
                return Self::internal()
                    .with_details(format!("Failed to deserialize SurrealError: {e}"));
            },
        };

        Self::Internal {
            data: Box::new(DatabaseErrorData {
                status: ErrorStatus::from(dto.code),
                code: "DATABASE_ERROR",
                message: Cow::Owned(dto.details),
                details: dto.description.map(Cow::Owned),
                help: dto.information.map(Cow::Owned),
            }),
        }
    }
}

use nx_error::prelude::*;

#[error]
pub(crate) enum AppError {
    #[error(message = "Configuration error", status = 500, code = "CONFIG_ERROR")]
    InvalidConfig,

    #[error(
        message = "Database operation failed",
        status = 500,
        code = "DATABASE_ERROR",
        source = surrealdb::Error,
    )]
    Database,

    #[error(message = "Migration failed", status = 500, code = "MIGRATION_ERROR")]
    Migration,

    #[error(message = "An internal system error occurred", status = 500, code = "INTERNAL_ERROR")]
    Internal,
}

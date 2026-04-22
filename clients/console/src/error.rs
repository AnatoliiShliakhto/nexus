use nx_error::error;

#[error]
pub enum ConsoleError {
    #[error(message = "Configuration error", code = "CON_CONFIGURATION_ERROR")]
    Config,

    #[error(message = "An unexpected internal error occurred", code = "CON_INTERNAL_ERROR")]
    Internal,
}

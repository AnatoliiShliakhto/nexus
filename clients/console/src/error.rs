use nx_error::prelude::*;

#[error]
pub enum ConsoleError {
    #[error(message = "Configuration error", code = "CON_CONFIGURATION_ERROR")]
    Config,

    #[error(message = "An unexpected internal error occurred", code = "CON_INTERNAL_ERROR")]
    Internal,
}

impl ConsoleError {
    pub(crate) fn emit(&self) {
        tracing::error!(
            code = %self.code(),
            details = %self.details().as_deref().unwrap_or(""),
            "{}", self.message(),
        );
    }
}

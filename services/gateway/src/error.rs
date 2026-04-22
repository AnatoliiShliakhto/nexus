use nx_error::prelude::*;

#[error]
pub enum GatewayError {
    #[transparent(nx_config::error::ConfigError)]
    Config,

    #[error(
        message = "Failed to initialize terminate signal",
        status = ErrorStatus::InternalServerError,
        code = "GW_SHUTDOWN_SIGNAL_INIT",
    )]
    Shutdown,

    Internal,
}

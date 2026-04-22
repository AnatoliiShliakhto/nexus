use nx_error::prelude::*;

#[error]
pub enum ConfigError {
    #[error(
        message = "Vault configuration is invalid",
        status = ErrorStatus::InternalServerError,
        code = "CONFIG_VAULT_BUILDER_ERROR",
        source = vaultrs::client::VaultClientSettingsBuilderError,
    )]
    VaultBuilder,

    #[error(
        message = "Failed to communicate with Vault",
        status = ErrorStatus::BadGateway,
        code = "CONFIG_VAULT_ERROR",
        source = vaultrs::error::ClientError,
    )]
    Vault,

    #[error(
        message = "Failed to load configuration",
        status = ErrorStatus::InternalServerError,
        code = "CONFIG_LOAD_FAILURE",
        source = config::ConfigError,
        help = "Verify that the config file exists and is readable.",
    )]
    Load,

    #[error(
        message = "Failed to load configuration file",
        status = ErrorStatus::InternalServerError,
        code = "CONFIG_LOAD_FAILURE",
        source = std::io::Error,
        help = "Check if the file path is correct and the application has read permissions.",
    )]
    Io,

    #[error(
        message = "Required environment variable is missing",
        status = ErrorStatus::InternalServerError,
        code = "CONFIG_ENV_MISSING",
        source = dotenvy::Error,
    )]
    Env,

    #[error(
        message = "The requested configuration key or file was not found",
        status = ErrorStatus::NotFound,
        code = "CONFIG_NOT_FOUND",
    )]
    NotFound,

    #[error(
        message = "Configuration validation failed",
        status = ErrorStatus::BadRequest,
        code = "CONFIG_INVALID",
    )]
    Invalid,
}

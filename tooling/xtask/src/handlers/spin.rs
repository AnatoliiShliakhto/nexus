use crate::error::AppError;
use crate::services::utils::run_command;

/// Spin up a local development server
pub(crate) fn run_spin_server() -> Result<(), AppError> {
    run_command("spin", &["up", "--runtime-config-file", "spin.config.toml"])
}

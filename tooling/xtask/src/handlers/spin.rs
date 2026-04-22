use crate::error::{AppError, AppErrorExt};
use crate::services::utils::{run_command, run_command_silent};
use std::fs;
use tracing::info;

/// Spin up a local development server
pub(crate) fn run_spin_server() -> Result<(), AppError> {
    resolve_secrets()?;
    run_command("spin", &["up", "--runtime-config-file", "spin.config.toml"])
}

fn resolve_secrets() -> Result<(), AppError> {
    info!("> Resolving secrets...");
    let secret_dir = std::env::current_dir()?.join(".secrets").join("database");

    if !secret_dir.exists() {
        return Err(AppError::internal()
            .with_details("Secrets directory not found")
            .with_help("Run `cargo x keys generate` to generate them"));
    }

    let private_key = fs::read_to_string(secret_dir.join("private_key.pem"))
        .with_help("Failed to read private key")?;
    let public_key = fs::read_to_string(secret_dir.join("public_key.pem"))
        .with_help("Failed to read public key")?;

    run_command_silent(
        "docker",
        &[
            "exec",
            "-e",
            "VAULT_TOKEN=dev-token",
            "-i",
            "vault",
            "vault",
            "kv",
            "put",
            "secret/nexus",
            &format!("database_private_key={private_key}"),
            &format!("database_public_key={public_key}"),
            &format!("database_user={}", env!("DATABASE_USER")),
            &format!("database_pass={}", env!("DATABASE_PASS")),
        ],
    )
    .with_help("Failed to write secrets to Vault. Run `cargo x dev up` to start infrastructure.")?;

    info!("> Secrets successfully resolved");
    Ok(())
}

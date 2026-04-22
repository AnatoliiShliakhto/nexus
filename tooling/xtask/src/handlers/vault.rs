use crate::domain::args::VaultAction;
use crate::error::{AppError, AppErrorExt};
use crate::services::utils::run_command_silent;
use dotenvy::var;
use std::fs;
use tracing::info;

pub(crate) fn handle_vault(action: &VaultAction) -> Result<(), AppError> {
    match action {
        VaultAction::Update => resolve_secrets(),
    }
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
            &format!(
                "VAULT_TOKEN={}",
                var("VAULT_TOKEN").unwrap_or_else(|_| "dev-token".to_owned())
            ),
            "-i",
            "vault",
            "vault",
            "kv",
            "put",
            &format!(
                "{}/nexus/database",
                var("VAULT_MOUNT")
                    .unwrap_or_else(|_| "nexus/database".to_owned())
                    .trim_end_matches('/')
            ),
            &format!("database_private_key={private_key}"),
            &format!("database_public_key={public_key}"),
            &format!(
                "database_user={}",
                var("DATABASE_USER").unwrap_or_else(|_| "root".to_owned())
            ),
            &format!(
                "database_pass={}",
                var("DATABASE_PASS").unwrap_or_else(|_| "root".to_owned())
            ),
        ],
    )
    .with_help("Failed to write secrets to Vault. Run `cargo x dev up` to start infrastructure.")?;

    info!("> Secrets successfully resolved");
    Ok(())
}

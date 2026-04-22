use crate::domain::args::KeysAction;
use crate::error::AppError;
use crate::services::crypto::KeyPair;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tracing::info;

pub(crate) fn handle_keys(keys_action: &KeysAction) -> Result<(), AppError> {
    match keys_action {
        KeysAction::Generate => rotate_keys(),
    }
}

pub(crate) fn rotate_keys() -> Result<(), AppError> {
    info!("> Rotating keys...");
    let secret_dir = std::env::current_dir()?.join(".secrets").join("database");

    if !secret_dir.exists() {
        fs::create_dir_all(&secret_dir).map_err(|e| {
            AppError::internal().with_details(format!("Failed to create .secrets dir: {e}"))
        })?;
    }

    let pair = KeyPair::generate()?;

    let private_path = secret_dir.join("private_key.pem");
    let private_pem = pair.private_key_pem()?;

    fs::write(&private_path, private_pem.as_str()).map_err(|e| {
        AppError::internal().with_details(format!("Failed to write private key: {e}"))
    })?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&private_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&private_path, perms)?;
    }

    let pub_path = secret_dir.join("public_key.pem");
    let pub_pem = pair.public_key_pem()?;

    fs::write(&pub_path, pub_pem).map_err(|e| {
        AppError::internal().with_details(format!("Failed to write public key: {e}"))
    })?;

    info!("> Keys successfully rotated in {secret_dir:?}");
    Ok(())
}

use crate::error::ConfigError;
use dotenvy::var;
use heck::{ToShoutySnakeCase, ToSnakeCase};
use std::fs;
use std::path::Path;

pub(crate) fn env_secret(key: &str) -> Result<String, ConfigError> {
    // 1. Try the standard Docker / Kubernetes secret path: /run/secrets/<key>
    let default_path = format!("/run/secrets/{}", key.to_snake_case());
    if Path::new(&default_path).exists() {
        let content = fs::read_to_string(&default_path)?;
        return Ok(content.trim().to_owned());
    }

    // 2. Try a custom path from the environment variable: <KEY>_PATH
    let shouty_key = key.to_shouty_snake_case();
    let path_env_key = format!("{shouty_key}_PATH");

    if let Ok(custom_path) = var(&path_env_key) {
        let path = Path::new(&custom_path);
        if path.exists() {
            let content = fs::read_to_string(path)?;
            return Ok(content.trim().to_owned());
        }
    }

    // 3. Fallback to direct environment variable: <KEY>
    let secret_val = env_var(&shouty_key)?;

    Ok(secret_val)
}

pub(crate) fn env_var(key: &str) -> Result<String, ConfigError> {
    var(key).map_err(ConfigError::from)
}

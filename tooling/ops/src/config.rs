use crate::error::AppError;
use heck::{ToShoutySnakeCase, ToSnakeCase};
use secrecy::SecretString;
use std::path::Path;
use std::{env, fs};

pub(crate) struct Config {
    pub url: SecretString,
    pub namespace: SecretString,
    pub database: SecretString,
    pub user: SecretString,
    pub pass: SecretString,
    pub public_key: SecretString,
}

impl Config {
    pub(crate) fn load() -> Result<Self, AppError> {
        if cfg!(debug_assertions) {
            Ok(Self {
                url: SecretString::from(env!("DATABASE_URL")),
                namespace: SecretString::from(env!("DATABASE_NAMESPACE")),
                database: SecretString::from(env!("DATABASE_NAME")),
                user: SecretString::from(env!("DATABASE_USER")),
                pass: SecretString::from(env!("DATABASE_PASS")),
                public_key: SecretString::from(include_str!(
                    "../../../.secrets/database/public_key.pem"
                )),
            })
        } else {
            Ok(Self {
                url: get_secret("DATABASE_URL")?,
                namespace: get_secret("DATABASE_NAMESPACE")?,
                database: get_secret("DATABASE_NAME")?,
                user: get_secret("DATABASE_USER")?,
                pass: get_secret("DATABASE_PASS")?,
                public_key: get_secret("DATABASE_PUBLIC_KEY")?,
            })
        }
    }
}

fn get_secret(key: impl AsRef<str>) -> Result<SecretString, AppError> {
    let key = key.as_ref();

    // 1. Try the standard Docker / Kubernetes secret path: /run/secrets/<key>
    let default_path = format!("/run/secrets/{}", key.to_snake_case());
    if Path::new(&default_path).exists() {
        let content = fs::read_to_string(&default_path).map_err(|e| {
            AppError::invalid_config()
                .with_details(format!("Secret file `{default_path}` read error: {e}"))
        })?;
        return Ok(SecretString::from(content.trim().to_owned()));
    }

    // 2. Try a custom path from the environment variable: <KEY>_PATH
    let shouty_key = key.to_shouty_snake_case();
    let path_env_key = format!("{shouty_key}_PATH");

    if let Ok(custom_path) = env::var(&path_env_key) {
        let path = Path::new(&custom_path);
        if path.exists() {
            let content = fs::read_to_string(path).map_err(|e| {
                AppError::invalid_config()
                    .with_details(format!("Custom secret file `{custom_path}` read error: {e}"))
            })?;
            return Ok(SecretString::from(content.trim().to_owned()));
        }
    }

    // 3. Fallback to direct environment variable: <KEY>
    let secret_val = env::var(&shouty_key).map_err(|_| {
        AppError::invalid_config().with_details(format!(
            "Secret not found. Checked: {default_path}, {path_env_key}, and env var {shouty_key}"
        ))
    })?;

    Ok(SecretString::from(secret_val))
}

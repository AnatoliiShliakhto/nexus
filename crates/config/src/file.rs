use crate::error::ConfigError;
use config::{Config, Environment, File};
use serde::de::DeserializeOwned;
use std::path::Path;
use tracing::info;

pub(crate) async fn load_file<T>(path: &Path) -> Result<T, ConfigError>
where
    T: DeserializeOwned + Clone,
{
    let effective_path = path.to_path_buf();

    let builder = Config::builder()
        .add_source(File::from(effective_path.as_path()).required(false))
        .add_source(
            // Env var overrides (e.g., NX__API__KEY)
            Environment::with_prefix("NX").separator("__").convert_case(config::Case::Snake),
        );

    info!("Loading config from `{}`", effective_path.display());

    let value = builder.build()?.try_deserialize::<T>()?;

    Ok(value)
}

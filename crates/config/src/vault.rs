use crate::environment::env_secret;
use crate::error::ConfigError;
use fxhash::FxHashMap;
use vaultrs::client::{Client, VaultClient, VaultClientSettingsBuilder};
use vaultrs::kv2;

pub(crate) async fn vault_variables(path: &str) -> Result<FxHashMap<String, String>, ConfigError> {
    let address = env_secret("VAULT_ADDR")?;
    let mount = env_secret("VAULT_MOUNT").unwrap_or_else(|_| "secret".to_owned());

    let mut settings_builder = VaultClientSettingsBuilder::default();
    settings_builder.address(address);

    if let Ok(cert) = env_secret("VAULT_CERT") {
        settings_builder.ca_certs(vec![cert]);
    }

    let settings = settings_builder.build()?;
    let mut client = VaultClient::new(settings)?;

    if let Ok(token) = env_secret("VAULT_TOKEN") {
        client.set_token(token.as_str());
    } else if let (Ok(role_id), Ok(secret_id)) =
        (env_secret("VAULT_ROLE_ID"), env_secret("VAULT_SECRET_ID"))
    {
        let auth = vaultrs::auth::approle::login(&client, "approle", &role_id, &secret_id).await?;
        client.set_token(&auth.client_token);
    } else {
        return Err(ConfigError::invalid()
            .with_details("Missing Vault credentials (VAULT_TOKEN or AppRole)"));
    }

    kv2::read::<FxHashMap<String, String>>(&client, &mount, path).await.map_err(ConfigError::from)
}

# Nexus Config

A robust, "Fortress-First" configuration management framework for Rust, built to handle complex secrets and layered
settings in modern cloud-native environments. This crate bridges the gap between static local configuration and dynamic
runtime secrets, ensuring that sensitive data is always protected and the application state is verified at compile-time.

## Key Features

* **Typestate-Safe Initialization:** Uses Rust's type system to ensure your application cannot access configuration
  until it is fully loaded and validated.
* **Fortress-First Secrets:** Implements a multi-stage discovery strategy (Files → Indirect Paths → Env) that wraps
  sensitive values in `SecretString` to prevent accidental logging or memory exposure.
* **Layered Configuration:** Seamlessly merges defaults, environment-specific files (TOML, JSON, YAML, etc.), and
  environment variable overrides using the `config-rs` engine.
* **Vault Integration:** Built-in support for HashiCorp Vault (KV2) with native authentication via Tokens or AppRole.
* **12-Factor Ready:** Designed for containers and orchestrators (Docker/K8s), supporting `/run/secrets` out of the box.

## Usage

### 1. Define and Load Configuration

Use the `Config` builder to transition from an uninitialized state to a loaded state.

```rust,ignore
use nx_config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppSettings {
    pub port: u16,
    pub database_url: String,
    pub log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), nx_config::error::ConfigError> {
    // 1. Initialize with your custom struct
    // 2. Load from directory (scans for default.toml, production.yaml, etc.)
    // 3. Merges with NX__ env overrides (e.g., NX__PORT=8080)
    let config = Config::init::<AppSettings>()
        .load("config")
        .await?;

    // Safe access to typed data
    println!("Listening on port: {}", config.get().port);

    Ok(())
}
```

### 2. Multi-Stage Secret Discovery

The `.secret()` method handles discovery automatically, ensuring your code remains environment-agnostic while following
high-security standards.

```rust,ignore
// Discovery Order for "db_password":
// 1. File: /run/secrets/db_password
// 2. Custom Path: Path found in DB_PASSWORD_PATH env var
// 3. Direct: DB_PASSWORD env var
let password: String = config.secret("db_password")?;
```

### 3. HashiCorp Vault Integration

Easily pull dynamic secrets from Vault and merge them into the internal cache.

```rust,ignore
let config = config
    .fetch_vault("kv/data/services/api-gateway")
    .await?;

// Now secrets from Vault are available via .secret()
let stripe_key: String = config.secret("stripe_api_key")?;
```

## Discovery Strategy Table

| Source      | Priority     | Logic / Path                                                  |
|-------------|--------------|---------------------------------------------------------------|
| Secret File | 1 (High)     | Reads from `/run/secrets/<snake_case_key>`                    |
| Custom Path | 2            | Reads from file at path defined in `<SHOUTY_KEY>_PATH`        |
| Direct Env  | 3 (Fallback) | Reads value directly from environment variable `<SHOUTY_KEY>` |

## Secrets & Vault Connection Strategy

The `.fetch_vault()` method follows the "Fortress-First" principle. It utilizes the internal env_secret discovery logic
to find Vault's own credentials. This allows you to securely inject `VAULT_TOKEN` or `VAULT_SECRET_ID` via Docker
Secrets, Kubernetes Sidecars, or standard environment variables.

### Secrets with Environment Variables Fallback for Vault

* `VAULT_URL`: The URL of your Vault server (e.g., `https://vault.internal:8200`).
* `VAULT_MOUNT`: The KV2 mount point (Defaults to `secret`).
* `VAULT_TOKEN`: Static token authentication.
* `VAULT_ROLE_ID` & `VAULT_SECRET_ID`: Used for AppRole authentication if no token is provided.
* `VAULT_CERT`: Optional CA certificate for TLS verification.

## Typestate Architecture

| State Marker   | Description                   | Available Methods                              |
|----------------|-------------------------------|------------------------------------------------|
| NotInitialized | Starting state.               | `init<T>()`, `from_file<T>()`                  |
| NoConfig       | Initialized but no file data. | `load()`, `var()`, `secret()`, `fetch_vault()` |
| WithConfig     | Fully loaded with typed data. | `get()`, `var()`, `secret()`, `fetch_vault()`  |

## Pro-Tip for Architects

By utilizing the `NX__` prefix for environment variables, you can override any nested configuration field without
changing your TOML files. For example, `NX__DATABASE__POOL_SIZE=20` will automatically override the value in your
`database.pool_size` configuration section. This allows for identical container images that adapt perfectly to Dev,
Staging, and Production environments.

## License

Copyright © 2026 Nexus Project. All rights reserved.
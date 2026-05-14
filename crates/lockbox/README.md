# Lockbox

A robust, **"Secure-by-Default"** credential management framework for Rust, designed to handle sensitive secrets across
diverse environments. `Lockbox` bridges the gap between various platform-specific secure enclaves (like macOS Keychain
or
Windows Vault) and modern cloud-native applications, ensuring your secrets are always encrypted at rest and accessed via
a unified, thread-safe API.

## Key Features

* **Platform-Agnostic API:** Write once, run anywhere. Seamlessly switches between macOS Keychain, Windows Credential
  Manager, Linux Secret Service, and Android SharedPreferences.
* **WASM/WASI Optimized:** Built-in support for SQLite-based storage in WASM environments (like `Spin` or `Nexus`),
  ensuring persistence even in virtualized file systems.
* **Type-Safe JSON Storage:** Beyond simple strings, `Lockbox` leverages `serde` to securely store and retrieve complex
  data structures as encrypted JSON.
* **Thread-Safe Architecture:** Uses `Arc` and internal synchronization to ensure that secret management is safe to use
  in highly concurrent async environments.
* **Zero-Configuration Native Discovery:** Automatically detects the best available hardware/OS-level storage for the
  current host without manual intervention.

## Usage

### 1. Basic Secret Management

Initialize Lockbox with a service name to namespace your credentials.

```rust
use nx_lockbox::Lockbox;

fn main() -> Result<(), nx_lockbox::error::LockboxError> {
    // Initialize with a service name (namespaces your secrets)
    let vault = Lockbox::new("nexus-gateway")?;

    // Store a sensitive token
    vault.set("api_key", "sk-proj-12345abcde")?;

    // Retrieve it back
    let token = vault.get("api_key")?;
    println!("Retrieved secret: {token}");

    Ok(())
}
```

### 2. Secure JSON Storage

Store complex structs (like session data or identity profiles) as encrypted blobs.

```rust
use nx_lockbox::Lockbox;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct SessionProfile {
    user_id: String,
    roles: Vec<String>,
    expires_at: u64,
}

fn save_session(lockbox: &Lockbox, profile: &SessionProfile) -> Result<(), nx_lockbox::error::LockboxError> {
    // Automatically serializes to JSON before encryption
    lockbox.set_json("current_session", profile)
}
```

### 3. Explicit Store Selection

In specialized environments, you can manually select which backend to use.

```rust,ignore
use nx_lockbox::Lockbox;

// Force the use of SQLite (useful for cross-platform CLI tools)
let vault = Lockbox::with_store("my-app", "sqlite")?;
```

## Supported Backends (NAMED_STORES)

| Name           | Platform       | Implementation                              |
|----------------|----------------|---------------------------------------------|
| keychain       | macOS          | Apple Keychain Services                     |
| windows        | Windows        | Windows Credential Manager (Vault)          |
| secret-service | Linux / BSD    | DBus Secret Service (Gnome Keyring/KWallet) |
| sqlite         | Cross-platform | Encrypted SQLite database                   |
| keyutils       | Linux          | Kernel Key Management Utilities             |
| protected      | iOS / macOS    | Apple Protected Data (Sandboxed)            |
| android        | Android        | SharedPreferences (Encrypted)               |
| sample         | Any            | In-memory store (Testing only)              |

## Pro-Tip for Architects

In **WASM/WASI** environments, `Lockbox` defaults to the `sqlite` store. For this to work in environments like Spin, ensure
you have a key-value or SQL component configured. This allows you to maintain the same "Secret Storage" API in your
microservices that you use in your native desktop CLI tools, providing a unified security boundary across your entire
stack.

## License

Copyright © 2026 Nexus Project. All rights reserved.
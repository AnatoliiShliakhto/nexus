#![allow(clippy::disallowed_types)]
use crate::error::LockboxError;
use keyring_core::{set_default_store, unset_default_store};
use std::collections::HashMap;

/// A list of built-in credential store backend identifiers.
///
/// These names are used with [`Lockbox::with_store`] to manually select a
/// specific storage implementation.
///
/// ### Common Platform Mapping:
/// - **macOS**: `keychain` (Native Keychain Services)
/// - **Windows**: `windows` (Windows Credential Manager)
/// - **Linux**: `secret-service` (DBus/Gnome Keyring) or `keyutils` (Kernel Keyring)
/// - **WASM/WASI**: `sqlite` (Persistent encrypted database)
pub const NAMED_STORES: [&str; 9] = [
    // Android Shared Preferences storage.
    "android",
    // macOS Keychain Services API.
    "keychain",
    // Linux Kernel Key Management Utilities (Keyutils).
    "keyutils",
    // Apple Protected Data storage (iOS/Sandboxed macOS).
    "protected",
    // An in-memory store for testing and development.
    "sample",
    // Linux/FreeBSD `DBus` Secret Service (via libdbus).
    "secret-service",
    // Linux/FreeBSD `DBus` Secret Service (via zbus, asynchronous).
    "secret-service-async",
    // Cross-platform SQLite-based encrypted storage.
    "sqlite",
    // Windows Credential Manager API.
    "windows",
];

/// Set the default store to one of the known stores (default configuration).
pub(crate) fn use_named_store(name: &str) -> Result<(), LockboxError> {
    if name.to_lowercase().as_str() == "sample" {
        use_sample_store(&HashMap::from([("persist", "true")]))
    } else {
        use_named_store_with_modifiers(name, &HashMap::new())
    }
}

/// Set the default store to one of the known stores (specified configuration).
///
/// The modifiers are passed to the store builder.
pub(crate) fn use_named_store_with_modifiers(
    name: &str,
    modifiers: &HashMap<&str, &str>,
) -> Result<(), LockboxError> {
    match name.to_lowercase().as_str() {
        "android" => use_android_native_store(modifiers),
        "keychain" => use_apple_keychain_store(modifiers),
        "keyutils" => use_linux_keyutils_store(modifiers),
        "protected" => use_apple_protected_store(modifiers),
        "sample" => use_sample_store(modifiers),
        "secret-service" | "secret-service-sync" => use_dbus_secret_service_store(modifiers),
        "secret-service-async" => use_zbus_secret_service_store(modifiers),
        "sqlite" => use_sqlite_store(modifiers),
        "windows" => use_windows_native_store(modifiers),
        _ => {
            let available = NAMED_STORES.join(", ");
            Err(LockboxError::invalid_store()
                .with_details_fn(|| format!("Received store name: '{name}'"))
                .with_help_fn(|| format!("Try one of the supported stores: {available}")))
        },
    }
}

/// Set the default store to the platform's OS-provided credential store.
#[allow(unused_variables)]
pub(crate) fn use_native_store(not_keyutils: bool) -> Result<(), LockboxError> {
    #[cfg(target_os = "android")]
    use_named_store("android")?;

    #[cfg(target_os = "macos")]
    use_named_store("keychain")?;

    #[cfg(target_os = "windows")]
    use_named_store("windows")?;

    #[cfg(target_os = "linux")]
    if not_keyutils {
        use_named_store("secret-service")?;
    } else {
        use_named_store("keyutils")?;
    }

    #[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
    use_named_store("secret-service")?;

    #[cfg(not(any(
        target_os = "android",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "windows",
    )))]
    use_named_store("sample")?;

    Ok(())
}

/// Set the default store to the `keyring-core::Sample` store.
pub(crate) fn use_sample_store(config: &HashMap<&str, &str>) -> Result<(), LockboxError> {
    use keyring_core::sample::Store;
    // Errors from keyring_core are automatically tunneled via #[transparent]
    set_default_store(Store::new_with_configuration(config)?);
    Ok(())
}

/// Use the macOS Keychain Services store.
#[allow(unused_variables)]
pub(crate) fn use_apple_keychain_store(config: &HashMap<&str, &str>) -> Result<(), LockboxError> {
    #[cfg(target_os = "macos")]
    {
        use apple_native_keyring_store::keychain::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(LockboxError::not_supported()
            .with_details("The Apple Keychain backend requires macOS")
            .with_help("Use 'secret-service' or 'keyutils' on Linux, or 'windows' on Windows"))
    }
}

/// Use the iOS/macOS Protected Data store.
#[allow(unused_variables)]
pub(crate) fn use_apple_protected_store(config: &HashMap<&str, &str>) -> Result<(), LockboxError> {
    #[cfg(target_os = "macos")]
    if std::env::var("APP_SANDBOX_CONTAINER_ID").is_ok() {
        use apple_native_keyring_store::protected::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    } else {
        Err(LockboxError::not_supported()
            .with_details("The macOS Protected Data store requires a provisioning profile")
            .with_help("Ensure the application is sandboxed and properly signed"))
    }

    #[cfg(target_os = "ios")]
    {
        use apple_native_keyring_store::protected::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        Err(LockboxError::not_supported()
            .with_details("Apple Protected Data store is only available on Apple platforms"))
    }
}

/// Use the Linux Keyutils store.
#[allow(unused_variables)]
pub(crate) fn use_linux_keyutils_store(config: &HashMap<&str, &str>) -> Result<(), LockboxError> {
    #[cfg(target_os = "linux")]
    {
        use linux_keyutils_keyring_store::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(LockboxError::not_supported()
            .with_details("The keyutils backend requires a Linux kernel"))
    }
}

/// Use the dbus-based Secret Service store via `libdbus`.
#[allow(unused_variables)]
pub(crate) fn use_dbus_secret_service_store(
    config: &HashMap<&str, &str>,
) -> Result<(), LockboxError> {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        use dbus_secret_service_keyring_store::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    {
        Err(LockboxError::not_supported()
            .with_details("The DBus Secret Service backend is only available on Linux and FreeBSD"))
    }
}

/// Use the dbus-based Secret Service store via `zbus`.
#[allow(unused_variables)]
pub(crate) fn use_zbus_secret_service_store(
    config: &HashMap<&str, &str>,
) -> Result<(), LockboxError> {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        use zbus_secret_service_keyring_store::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    {
        Err(LockboxError::not_supported()
            .with_details("The zbus Secret Service backend is only available on Linux and FreeBSD"))
    }
}

/// Use the Windows Credential store.
#[allow(unused_variables)]
pub(crate) fn use_windows_native_store(config: &HashMap<&str, &str>) -> Result<(), LockboxError> {
    #[cfg(target_os = "windows")]
    {
        use windows_native_keyring_store::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(LockboxError::not_supported()
            .with_details(
                "The 'windows' store backend relies on the Windows Credential Manager API",
            )
            .with_help(
                "Switch to 'secret-service' (DBus) or 'keyutils' (Kernel) for Linux environments",
            ))
    }
}

/// Use the Android Shared Preferences store.
#[allow(unused_variables)]
pub(crate) fn use_android_native_store(config: &HashMap<&str, &str>) -> Result<(), LockboxError> {
    #[cfg(target_os = "android")]
    {
        use android_native_keyring_store::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(target_os = "android"))]
    {
        Err(LockboxError::not_supported()
            .with_details("Android Shared Preferences store is only available on Android"))
    }
}

/// Use a cross-platform encrypted sqlite (Turso) database.
#[allow(unused_variables)]
pub(crate) fn use_sqlite_store(config: &HashMap<&str, &str>) -> Result<(), LockboxError> {
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        use db_keystore::DbKeyStore;
        set_default_store(DbKeyStore::new_with_modifiers(config)?);
        Ok(())
    }
    #[cfg(any(target_os = "ios", target_os = "android"))]
    {
        Err(LockboxError::not_supported().with_details(
            "The SQLite store is not currently supported on mobile platforms (iOS/Android)",
        ))
    }
}

/// Release the current default store.
#[allow(dead_code)]
pub(crate) fn release_store() {
    unset_default_store();
}

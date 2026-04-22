# Nexus Error Macros

A zero-overhead procedural macro engine for the Nexus
error-handling framework. This crate provides the `#[error]` attribute, designed to build
"Fortress-First" systems where security, observability, and performance are non-negotiable.

## Overview

In mission-critical `WebAssembly` (WASI/Spin) and high-load Rust environments, standard backtraces
are
often too heavy or restricted. `nx-error-macros` replaces them with a Static Metadata Chain.
Unlike traditional error macros, `nx-error-macros` performs metadata resolution at compile-time,
ensuring that your error handling adds zero runtime latency while providing enterprise-grade
telemetry.

## Features

1. Zero-Cost Transparent Delegation\
   The `#[transparent(T)]` attribute allows your error enum to wrap upstream errors while
   "inheriting" their metadata (status, code, message) automatically.
    * **Inheritance:** If the inner error is `DatabaseError::NotFound` (404), your `ServiceError`
      will also report 404.
    * **Override:** You can still override specific fields:
      `#[transparent(source = DatabaseError, status = 403)]`.
2. Smart Attribute Parser\
   Supports expressive shorthands to keep your code clean:
    * **Direct Status:** `#[error(404)]` or `#[error(ErrorStatus::Conflict)]`.
    * **Inference:** Automatically generates machine-readable CODE and human-readable Messages from
      variant names (e.g., UserNotFound → USER_NOT_FOUND / "User not found").
3. Native Wasm Optimization\
   Offline-First: Designed for air-gapped and restricted environments.
    * **Small Binary Footprint:** Generates efficient code without unnecessary dependencies or heavy
      runtime logic.
    * **Cow-Powered:** Uses `Cow<'static, str>` to minimize heap allocations and support static
      strings by default.
4. Effortless Integration
    * **Automatic `From<T>`:** Injects conversions for source types (like `std::io::Error`),
      enabling
      the `?` operator immediately.
    * **Result Extensions:** Provides a fluent API (`.with_message()`, `.with_details()`,
      `.with_help()`) to enrich errors as they bubble up.

## Generated API

For an enum pub enum `UserError`, the macro provides:

* **Static Constructors:** `UserError::not_found()` (snake_case conversion).
* **Metadata Accessors:** Implementation of the `ErrorMetadata` trait.
* **Fluent Mutators:** `.with_message()`, `.with_details()`, and `.with_help()`.
* **Lazy Mutators:** `_fn` variants (e.g., `.with_message_fn(|| ... )`) for expensive string
  formatting that only runs on the error path.

### Attributes Reference

Configure your variants using the `#[error(...)]` and `#[transparent(...)]` attributes:

| Property      | Format | Description                                         |
|---------------|--------|-----------------------------------------------------|
| transparent   | Type   | Delegates all metadata to the wrapped type T.       |
| status / kind | Expr   | Numeric code (404) or path (ErrorStatus::NotFound). |
| code          | String | Machine slug. Defaults to SHOUTY_SNAKE_CASE.        |
| message       | String | Human description. Defaults to Sentence case.       |
| source        | Type   | Upstream type for From conversion and chaining.     |
| help          | String | Troubleshooting advice for end-users or logs.       |
| target        | String | Troubleshooting advice for end-users or logs.       |

## Example

```rust,ignore
use nx_error::error;

#[error]
pub enum ServiceError {
    // 1. Transparent: Inherits 404/NotFound from the inner DB error
    #[transparent(DatabaseError)]
    Db,

    // 2. Shorthand: Set status only, infer Code and Message
    #[error(401)]
    Unauthorized,

    // 3. Wrapping External: Wrap std::io, provide custom context
    #[error(status = 500, message = "Disk access failed", source = std::io::Error)]
    FileSystem,

    // 4. Manual: Full control over all fields
    #[error(
        status = 409, 
        code = "USER_CONFLICT", 
        message = "User already exists",
        help = "Try using a different email address."
    )]
    Conflict,
}

// Usage with Result Extension
fn register_user() -> Result<(), ServiceError> {
    check_db().with_message("Database check failed during registration")
             .with_help("Verify database connectivity in the dashboard")
}
```

## Safety & Visibility

The macro automatically handles `#[allow(unreachable_pub)]`, making it safe to use within nested test
modules or complex crate hierarchies without triggering visibility warnings.

## License

Copyright © 2026 Nexus Project. All rights reserved.
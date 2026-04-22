# Nexus Error

An error-handling framework tailored for `WASI` and mission-critical
Rust systems. This crate replaces heavy standard library backtraces with a Static Metadata Chain,
turning technical failures into categorized, traceable domain errors that benefit both developers
(tree reports) and machines (JSON telemetry).

## Key Features

* **Zero-Boilerplate Macros:** Use `#[error]` for automatic `From` conversions, snake_case
  constructors, and metadata implementation.
* **Zero-Cost Transparency:** Use `#[transparent(T)]` to wrap upstream errors while inheriting their
  status, code, and messages at compile-time.
* **Semantic Status Mapping:** Every error maps to an ErrorStatus (aligned with HTTP codes) for
  seamless integration with Axum or Spin.
* **Hierarchical Reporting:** A built-in tree formatter for visualizing deep cause chains and
  troubleshooting advice.
* **Wasm-Optimized:** Designed with `Cow<'static, str>` and zero-dependency cores to minimize memory
  footprint in Spin/WASI component architectures.

## Usage

### 1. Advanced Domain Error Definition

The `#[error]` macro supports Convention over Configuration. It infers machine codes and human
messages from variant names automatically.

```rust 
use nx_error::prelude::*;

#[error]
pub enum GatewayError {
    // 1. Transparent Delegation: Inherits status and code from DatabaseError
    #[transparent(DatabaseError)]
    Db,

    // 2. Full manual specification
    #[error(status = 401, message = "Unauthorized access", code = "AUTH_FAIL")]
    Auth,

    // 3. Shorthand: Status code only. Code becomes "NOT_FOUND", Message: "Not found"
    #[error(404)]
    NotFound,

    // 4. Automatic Source transformation
    #[error(status = ErrorStatus::InternalStatusError, source = std::io::Error)]
    NetworkIssue,

    // 5. Complete default: Status=500, Code="INTERNAL_FAILURE", Message="Internal failure"
    InternalFailure,
}
``` 

### 2. Contextual Enrichment

Use the `ErrorMetadataExt` trait to layer operational details as errors bubble up the stack without
losing the original root cause.

```rust 
use nx_error::ErrorMetadataExt;

fn get_config() -> Result<String, GatewayError> {
    std::fs::read_to_string("config.json")
        .with_details("Missing required system profile")
        .with_help("Check if config.json exists in the root directory")?;
    Ok("data".into())
}
```

### 3. Multi-Source Conversion (The DX Power-Up)

In complex systems, a single domain-level error variant often stems from multiple lower-level technical failures. The
`from` attribute allows the macro to automatically generate `impl From<T>` for an array of types, funneling them through the
primary `source`.

```rust 
#[error]
pub enum MyError {
    #[transparent(
        source = nx_http::error::Error,
        // Automatically generates From for all specified types
        from = [
            nx_http::url::ParseError,
            nx_http::request::RequestError,
        ],
    )]
    Http,
}
```

## Inspection & Reporting

### Human-Readable Tree Reports

The `.report()` method generates an ASCII tree that is far more readable than a standard Debug dump,
especially in air-gapped or CLI environments.

```rust,ignore
× [DB_CONN_LOST]: Connection lost
    Status: 503 Service Unavailable | Target: database-service
  │
  ├─ Caused by:
  │  1: Timed out waiting for connection pool
  │  2: No route to host (os error 113)
  │
  ├─ Details:
  │  Failed to connect to cluster: production-01
  │
  ╰─ Help: Restart the database proxy or check the VPC security group.
``` 

### Structured JSON Telemetry

Enable the `json` feature to serialize errors for Grafana Loki, ELK, or Datadog pipelines.

```rust,ignore 
// Simple JSON for API responses
let json = err.to_json_string();

// Detailed JSON for internal logging (includes full cause chain)
let telemetry = err.to_detailed_json_string();
```

## Architecture: The Metadata Chain

| Method           | Type                | Description                                             |
|------------------|---------------------|---------------------------------------------------------|
| `status()`       | `ErrorStatus`       | HTTP-compatible code (e.g., 400, 404, 500).             |
| `code()`         | `&'static str`      | Unique machine-readable slug (e.g., AUTH_EXPIRED).      |
| `message()`      | `Cow<'static, str>` | Human-readable primary description.                     |
| `details()`      | `Option<Cow>`       | Dynamic operational context (IDs, paths).               |
| `help()`         | `Option<Cow>`       | Actionable troubleshooting advice.                      |
| `error_source()` | `Option<&dyn>`      | Access to the next metadata-capable error in the chain. |

## Fluent Builder API (`ErrorMetadataExt`)

All extension methods support method-chaining and are implemented for both `ErrorMetadata` types and
`Result<T, E>`:

* `.with_message(...)`: Overwrites the default error message.
* `.with_details(...)`: Attaches high-level operational context.
* **Lazy Mutators:** `.with_message_fn()`, `.with_details_fn()`, and `.with_help_fn()` use closures
  to ensure expensive string formatting only occurs on the error path (Zero-Cost on success).

```rust,ignore
let result = get_user_config()
    .with_details("Failed to retrieve user configuration")
    .with_help_fn(|| format!("Documentation: {}", "https://example.com/docs"));
```

## Pro-Tip for Architects

By using `#[transparent]`, you create a Zero-Cost Error Tunnel. Your high-level services don't need
to know the specifics of database or auth failures to report the correct HTTP status code to the
client. This maintains strict domain boundaries while preserving full observability.

## License

Copyright © 2026 Nexus Project. All rights reserved.
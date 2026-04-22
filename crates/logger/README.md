# Nexus Logger

A centralized logging and observability crate for Nexus services and clients.

It provides a unified way to configure:
- console logging
- rotating file logging
- non-blocking output
- environment-based filtering
- optional OpenTelemetry integration
- optional profiling support
- field redaction for sensitive values

## Features

- **Typestate builder** for safer initialization
- **Non-blocking I/O** via background worker guards
- **Console and file layers**
- **Daily/hourly/etc. log rotation**
- **JSON or compact human-readable output**
- **Environment-driven overrides**
- **Field filtering/redaction**
- **WASM-aware compilation**
- **Optional OpenTelemetry tracing**
- **Optional tokio-console subscriber profiling**

## Installation

Add it as a workspace dependency from your monorepo:

```rust,toml 
[dependencies] 
nx-logger.workspace = true
```

If you need OpenTelemetry support:

```rust,toml 
[dependencies] 
nx-logger = { workspace = true, features = ["opentelemetry-otpl"] }
```

If you need profiling support:

```rust,toml 
[dependencies] 
nx-logger = { workspace = true, features = ["profiling"] }
```

## Quick start

```rust,ignore
rust use nx_logger::{LevelFilter, Logger};
fn main() { 
    let _logger = Logger::builder() 
        .name("nx-gateway") 
        .console(true) 
        .level(LevelFilter::INFO) 
        .init() 
        .expect("failed to initialize logger");
    
    tracing::info!("logger initialized");
}
```


## Builder overview

The logger uses a typestate builder to prevent invalid configuration at compile time.

Typical flow:

1. create a builder
2. set a service name
3. optionally configure console/file/format options
4. call `init()`

Example with file logging:

```rust,ignore
use nx_logger::{LevelFilter, Logger, Rotation};

fn main() { 
    let _logger = Logger::builder() 
        .name("nx-console") 
        .console(true) 
        .path("logs") 
        .rotation(Rotation::DAILY) 
        .max_files(10) 
        .level(LevelFilter::DEBUG) 
        .init() 
        .expect("failed to initialize logger"); 
}
```


## Configuration

The builder provides defaults, but you can override behavior with environment variables.

### Supported environment variables

- `LOG_JSON`  
  Enable JSON output when set to `true` or `1`.

- `LOG_ANSI`  
  Enable ANSI colors in console output when set to `true` or `1`.

- `LOG_SPANS`  
  Include active span lists in JSON logs when set to `true` or `1`.

- `LOG_IGNORE_FIELDS`  
  Comma-separated list of fields to redact from formatted output.  
  Example:
  ```bash
  LOG_IGNORE_FIELDS=password,token,secret
  ```

- `RUST_LOG`  
  Standard tracing directives.  
  Example:
  ```bash
  RUST_LOG=info,nx_gateway=debug
  ```
  
- `OTEL_EXPORTER_OTLP_ENDPOINT`
  OpenTelemetry endpoint
  Example: 
  ```bash
  OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"
  ```

- `OTEL_SERVICE_NAME`
  OpenTelemetry service name
  Example:
  ```bash
  OTEL_SERVICE_NAME="http://localhost:4318"
  ```

- `OTEL_RESOURCE_ATTRIBUTES`
  OpenTelemetry 
  Example:
  ```bash
  OTEL_RESOURCE_ATTRIBUTES="http://localhost:4318"
  ```


## Console output

Console logging can be enabled or disabled independently.

Example:

```rust,ignore
let _logger = Logger::builder() 
    .name("nx-service") 
    .console(true) 
    .init() 
    .expect("failed to initialize logger");
```


## File output

On non-WASM targets, file logging is supported with rotation.

Example:

```rust,ignore
use nx_logger::{LevelFilter, Logger, Rotation};

let _logger = Logger::builder() 
    .name("nx-service") 
    .console(true) 
    .path("logs") 
    .rotation(Rotation::DAILY) 
    .max_files(30) 
    .level(LevelFilter::INFO) 
    .init() 
    .expect("failed to initialize logger");
```


### Notes

- Log files are written asynchronously.
- Keep the returned `Logger` value alive for the whole program lifetime.
- Dropping the logger flushes buffers and stops worker threads gracefully.

## OpenTelemetry

If the feature is enabled, the crate can attach an OpenTelemetry tracing layer.

```toml
[dependencies] 
nx-logger = { workspace = true, features = ["opentelemetry-otlp"] }
```

Then configure it from your application bootstrap as needed.

## Profiling support

If the profiling feature is enabled on supported targets, the crate can activate a console subscriber for tracing/profiling workflows.

```toml
[dependencies] 
nx-logger = { workspace = true, features = ["profiling"] }
```


## WASM support

The crate is designed to compile cleanly for WASM targets.

Behavior differences:
- file logging is disabled on `wasm32`
- only console-style logging paths are available
- platform-specific appender and worker guard logic is conditionally compiled

## Error handling

Initialization returns `Result<Logger, LoggerError>`.

Common failure reasons:
- empty or invalid logger name
- invalid configuration
- invalid `env_filter`
- file appender initialization failure
- global subscriber already initialized

## Logger lifetime

The returned `Logger` handle must be kept alive.

If the handle is dropped too early, background logging may stop before all messages are flushed.

## Field redaction

The logger can suppress selected fields in formatted output.

This is useful for values like:
- `password`
- `token`
- `secret`
- `details`
- `backtrace`

Default ignored fields are configured by the crate, and can be overridden with `LOG_IGNORE_FIELDS`.

## Example in a service

```rust,ignore
use nx_logger::{LevelFilter, Logger};

pub fn init_logging() -> Logger { 
    Logger::builder() 
        .name("nx-gateway") 
        .console(true) 
        .level(LevelFilter::INFO) 
        .init() 
        .expect("failed to initialize logger") 
}
```


## Recommended usage

For most services:

- use `LevelFilter::INFO` in production
- enable `console(true)` during development
- use file logging for long-running services
- keep the returned `Logger` in `main()`

## License

Copyright © 2026 Nexus Project. All rights reserved.
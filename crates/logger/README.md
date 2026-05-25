# Nexus Logger

A centralized logging and observability crate for the Nexus ecosystem.

Designed for high-throughput, low-latency distributed systems, `nx-logger` provides a unified, zero-cost hot path for structured logging. It seamlessly integrates non-blocking I/O, environment-based filtering, field redaction, and optional OpenTelemetry tracing, all while maintaining WASM-native compatibility.

## Features

* **Zero-Cost Hot Path:** Stack-allocated JSON formatting using `SmallVec` and `SmolStr` to eliminate heap allocations during standard logging.
* **Non-Blocking I/O:** Dedicated background worker guards ensure logging never blocks your async tokio runtime.
* **Typestate Builder:** Compile-time validation prevents invalid logger configurations.
* **Advanced Field Redaction:** Automatically suppresses sensitive data (passwords, tokens, secrets) with zero runtime overhead.
* **Wasm-Native:** Compiles cleanly to `wasm32-unknown-unknown` and `wasm32-wasi`.
* **Telemetry & Profiling:** Out-of-the-box support for `opentelemetry-otlp` and `tokio-console`.
* **Telemetry, Metrics & Profiling:** Out-of-the-box support for the complete OpenTelemetry trinity (Traces, Logs, and Metrics via `opentelemetry-otlp`) and task profiling via `tokio-console`.

## Performance & Zero-Cost Architecture

`nx-logger` is engineered to handle millions of logs per second without creating GC pauses or lock contention in highly concurrent environments (like Axum/Tokio).

### Heap Profiling (DHAT)

I continuously profile the hot path using DHAT. When logging static strings and primitives:

* **0** Heap Allocations (`malloc`) per log event.
* All JSON serialization and field filtering happen directly on the stack.

### Concurrent Scaling (Criterion)

Async benchmarks demonstrate linear scalability with Tokio workers. Because the formatter is lock-free, adding more concurrent threads actually decreases the time spent per log.

| Concurrency | Total Logs | Batch Time | Time per Log | Scaling Efficiency |
|-------------|------------|------------|--------------|--------------------|
| 1 Worker    | 10         | 2.92 µs    | ~290 ns      | Base Speed         |
| 4 Workers   | 40         | 7.54 µs    | ~188 ns      | +35%               |
| 8 Workers   | 80         | 11.09 µs   | ~138 ns      | +52%               |

*Metrics captured on an 8-core CPU. Reproduce locally via `cargo bench -p nx-logger`.*

## Installation

Add it to your project:
```toml
[dependencies] 
nx-logger = "0.1.0" # Replace it with actual version
```

### Feature Flags

### Feature Flags

* `opentelemetry`: Enables exporting tracing spans and structural logs to OpenTelemetry collectors (via gRPC/Tonic).
* `metrics`: Activates the OpenTelemetry `MeterProvider` and a periodic metric reader, automatically enabling the base `opentelemetry` feature.
* `profiling`: Activates a console subscriber for tracing/profiling Tokio tasks via `tokio-console`.

## Quick Start

The logger uses a typestate builder pattern. The returned `Logger` handle must be kept alive for the lifetime of the application. Dropping it gracefully flushes buffers and shuts down the background I/O threads.

```rust
use nx_logger::{LevelFilter, Logger};

fn main() { 
    // Initialization returns a worker guard. 
    // Bind it to a variable to keep background threads alive!
    let _logger = Logger::builder() 
        .name("nx-gateway") 
        .console(true)
        .json(true)
        .level(LevelFilter::INFO) 
        .init() 
        .expect("Failed to initialize logger");
    
    tracing::info!(
        service = "gateway",
        "Logger successfully initialized and ready."
    );
}
```

## Configuration

### File Output & Rotation (Non-WASM)

File logging is processed asynchronously with built-in rotation rules.

```rust
use nx_logger::{LevelFilter, Logger, Rotation};

fn main() {
    let _logger = Logger::builder()
        .name("nx-service")
        .console(false)
        .path("/var/log/nexus")
        .rotation(Rotation::DAILY)
        .max_files(30)
        .level(LevelFilter::DEBUG)
        .init()
        .unwrap();
}
```

### Environment Overrides

While the builder sets defaults, behavior can be dynamically overridden without recompilation using environment variables:

| Variable          | Description                            | Example                 |
|-------------------|----------------------------------------|-------------------------|
| RUST_LOG          | Standard tracing directives.           | "info,nx_gateway=debug" |
| LOG_JSON          | Force JSON formatting (true/1).        | 1                       |
| LOG_ANSI          | Enable ANSI colors for console output. | true                    |
| LOG_SPANS         | Include active tracing spans in JSON.  | true                    |
| LOG_IGNORE_FIELDS | Comma-separated fields to redact.      | "password,token,secret" |

### OpenTelemetry Variables:

* `OTEL_EXPORTER_OTLP_ENDPOINT`="http://localhost:4318"
* `OTEL_EXPORTER_OTLP_INSECURE`="true"
* `OTEL_SERVICE_NAME`="nx-gateway"
* `OTEL_RESOURCE_ATTRIBUTES`="deployment.environment=production"
* `OTEL_METRIC_EXPORT_INTERVAL`="30000" *(Export interval in milliseconds)*

## Field Redaction (Security-First)

`nx-logger` includes a zero-cost field redaction layer. By default, it suppresses sensitive keys to prevent credentials from leaking into your aggregators (Datadog, ELK, etc.).

Default ignored fields include: `password`, `token`, `secret`, `authorization`. You can override this list via the `LOG_IGNORE_FIELDS` environment variable.

## WebAssembly (WASM) Support

The crate is designed to compile cleanly for WASM targets (`wasm32-unknown-unknown` and `WASI P2`).

**Behavior on WASM:**

* File logging features are automatically disabled.
* Platform-specific appender and worker guard logic is conditionally compiled out.
* Console-style logging routes safely to browser/WASI host stdout.
* **OTLP Metrics & Tracing:** The `opentelemetry` and `metrics` features are intended for native targets (`not(target_arch = "wasm32")`) due to their reliance on background threads and gRPC network stacks (`tonic`).

## License

Copyright © 2026 Nexus Project. All rights reserved.
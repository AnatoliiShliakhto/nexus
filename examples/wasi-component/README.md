# Nexus WASI Component

A WASI-HTTP Component reference implementation built on the `nx-http` framework. This template serves
as an architectural blueprint for creating robust, observable WebAssembly services that adhere to the Nexus API 
standards.

## Architecture

This component is built using the WASI Component Model (P2). It implements the `wasi:http/incoming-handler` interface,
making it portable across any WASI-compliant runtime, including `Wasmtime`, `Spin`, and `wasmCloud`.

The `nx-http` stack provides a "Guard Layer" that abstracts low-level WIT-generated bindings, allowing you to work with
the standard Rust `http` crate ecosystem.

## Key Features

### Zero-Boilerplate WASI Bridge

The `#[wasi_service]` macro seamlessly maps raw WASI types (like `IncomingRequest`) to the idiomatic
`http::Request<Bytes>`.
This allows for clean, synchronous business logic while the macro manages the underlying WASI-HTTP state machine.

### Built-in Observability

* **Trace Continuity:** Automatically extracts W3C traceparent headers to join existing distributed trace chains within
  the Nexus ecosystem.
* **Header Augmentation:** Injects an `x-trace-id` into every outgoing response, ensuring seamless correlation in Nexus
  monitoring dashboards.
* **Instrumentation:** Initializes a tracing span for the entire request lifecycle.

### Declarative Error Normalization

Integrates with `nx-error` to ensure that every failure in your business logic results in a machine-readable JSON
contract rather than a raw string or a generic 500 error.

**Standardized JSON Error Response:**
```json
{
  "status": 500,
  "code": "WASI_COMPONENT_INTERNAL_ERROR",
  "message": "Internal processing failure"
} 
```

## License

Copyright © 2026 Nexus Project. All rights reserved.
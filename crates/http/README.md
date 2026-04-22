# Nexus HTTP

A  Wasm-native toolkit designed for building robust, observable HTTP services
within the Nexus ecosystem.
`nx-http` serves as the foundation for modern Wasm development, consolidating telemetry,
standardized error handling, and runtime-specific integration logic into a single crate optimized
for `wasm32-wasip2` and `wasm32-wasip3` targets.

## Features

* **Wasm-Native Observability:** Fully optimized for Wasm runtimes, featuring minimal-overhead
  logging and telemetry.
* **Distributed Tracing (W3C):** Native support for traceparent extraction, propagation, and
  injection, ensuring full trace continuity across service boundaries.
* **Unified Error Pipeline:** Enforces structured JSON error responses via `nx-error`, preventing
  leakage of sensitive internal state.
* **Runtime-Agnostic Macros:** Seamless integration with Spin SDK (async) and WASI-HTTP
  (sync/Component Model) via a unified macro interface.
* **Security-First Logging:** Built-in field filtering to automatically scrub sensitive tokens,
  passwords, and PII from logs.

## Feature Matrix

| Feature  | Description                                               | Dependencies       |
|----------|-----------------------------------------------------------|--------------------|
| spin     | Integration with Fermyon Spin SDK v2/v3.                  | spin-sdk           |
| unstable | Experimental support for WASIp3 unstable features.        | spin-sdk, http     |
| wasi     | Direct WASI-HTTP component handlers and conversion logic. | http, bytes, serde |

## Usage

### Declarative Routing (The Recommended Way)

The `router!` macro (re-exported from nx-http-macros) allows you to define routes using a clean, pattern-matching syntax.
It integrates seamlessly with nx-http error types.

```rust,ignore
use nx_http::{router, spin_service};

#[spin_service]
async fn handle_request(req: Request, state: AppContext) -> Result<Response, MyError> {
    router!(
        req,
        // Static segments
        get  ["health"] => Ok(Response::new(200, "OK")),
        
        // Path parameters (captured as &str)
        get  ["user", id] => get_user(id, &state).await,
        
        // Tail matching for proxies or assets
        get  ["static", ..path] => serve_assets(path),
        
        // POST with dependency injection
        post ["login"] => auth::login(&req, &state).await,
        
        // Global 404 handler via your error enum
        _ => Err(MyError::not_found())
    )
}
```

### The Declarative Approach (Recommended)

Use `nx-http-macros` to automatically handle telemetry initialization, span creation, and error-to-JSON transformation
in one line.

```rust,ignore
#[nx_http::error::error]
pub enum MyError {
    #[error(source = http::Error, code = 400, kind = "BAD_GATEWAY")]
    Http,
}

// For Spin (Async)
#[nx_http::spin_service]
async fn handle_request(req: Request) -> Result<impl IntoResponse, MyError> {
    Ok(Response::builder().status(200).body("Hello Spin!").build())
}

// For WASI Components (Sync)
#[nx_http::wasi_service]
fn handle_wasi(req: Request<Bytes>) -> Result<Response<Bytes>, MyError> {
    Ok(Response::new(200, Bytes::from("Hello WASI!")))
}
```

### Manual Context Propagation

For complex scenarios where you need to manually propagate trace context to upstream requests:

```rust,ignore
fn call_upstream(mut req: http::Request<Bytes>) {
    let ctx = req.header_value("traceparent").map_or_else(TraceContext::new, TraceContext::from);

    // Inject traceparent into outgoing headers
    req.headers_mut().insert("traceparent", ctx.child().traceparent().parse().unwrap());
}
```

### Distributed Tracing

Manually propagate context to upstream services:

```rust,ignore
fn call_upstream(mut req: http::Request<Bytes>) {
    let span = req.traced_span();
    let ctx = req.trace_context();
}
```

### Advanced Request Proxying

The `ProxyRequest` builder provides a secure way to forward requests to upstream services, handling RFC 2616
(Hop-by-Hop) headers and path rebasing automatically.

```rust,ignore
use nx_http::spin_tools::proxy::{ProxyRequestExt, ProxyRequestError};

async fn proxy_handler(req: Request) -> Result<Response, ProxyRequestError> {
    // Transparently forward to internal microservice
    let outbound_req = req.proxy_to("https://inventory.internal.svc")
        .strip_prefix("/api/v1/proxy") // Omit the gateway prefix
        .header("X-Proxy-Source", "Nexus-Gateway") // Inject custom metadata
        .exclude_header("Cookie") // Scrub sensitive client state
        .strict_allowlist() // Forward only Content-Type, Length, and Accept
        .build()?;

    Ok(spin_sdk::http::send(outbound_req).await?)
}
```

## Environment Configuration

Control observability behavior via standard environment variables:

| Variable          | Default               | Description                                       |
|-------------------|-----------------------|---------------------------------------------------|
| RUST_LOG          | info                  | Filter level (e.g., debug, warn, my_crate=trace)  |
| LOG_JSON          | true                  | Set to 0 or false for human-readable compact logs |
| LOG_ANSI          | false                 | Enables colored output (compact mode only)        |
| LOG_IGNORE_FIELDS | password,backtrace... | Comma-separated list of fields to scrub from logs |
| DETAILED_ERROR    | false                 | Enables detailed error logging                    |

## Traceparent Format

The generated `traceparent` follows W3C version 00: `00-{trace_id}-{parent_id}-{flags}`

## License

Copyright © 2026 Nexus Project. All rights reserved.
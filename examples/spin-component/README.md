# Nexus Spin Component

A Fermyon Spin template demonstrating the "Golden Path" for Wasm-native microservices. Leveraging the
`nx-http` SDK, this template offloads distributed tracing, proxy logic, and error serialization to a declarative macro
layer.

## Key Parts

### The Async Service Handler

The `#[spin_service]` macro acts as a high-level middleware that manages the request lifecycle:

* **W3C Trace Context:** Automatically extracts traceparent from headers and initializes the tracing subscriber.
* **Response Instrumentation:** Injects `x-trace-id` into all outgoing responses for easy log correlation.
* **Error Interception:** Automatically transforms `Result::Err` into a standardized Nexus JSON response, preventing
  internal state leakage.

```rust,ignore
#[nx_http::spin_service]
async fn handle(request: Request) -> Result<impl IntoResponse, SpinComponentError> {
    // 1. Safe environment access
    let target_url = get_spin_var("target_url", Some("http://some.spin.internal"))?;
    
    // 2. Fluent proxying with automatic header scrubbing
    let outbound = request.proxy_to(target_url).build()?;
    
    // 3. Async WASI-HTTP outbound call
    let response = nx_http::spin_sdk::http::send(outbound).await?;

    Ok(response)
}
```

### Type-Safe Proxying

Instead of manual request cloning, the template uses the `ProxyRequest` builder. It handles **RFC 2616 Hop-by-Hop**
header removal, path rebasing, and security allowlists out of the box.

### Hierarchical Error System

Errors are defined in `error.rs` using the `#[error]` macro. This ensures a consistent, machine-readable JSON contract for frontend clients:

**Example JSON Error Response:**
```json
{
  "status": 503,
  "code": "SPIN_COMPONENT_UPSTREAM_UNAVAILABLE",
  "message": "Upstream service is unreachable or failed to respond"
}
```

### Project Structure

* `src/main.rs`: Async handler logic and service orchestration.
* `src/error.rs`: Centralized error definitions with `#[transparent]` mapping for submodules.
* `spin.toml`: Component configuration and allowed outbound hosts.

## License

Copyright © 2026 Nexus Project. All rights reserved.
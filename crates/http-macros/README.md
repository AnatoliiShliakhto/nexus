# Nexus HTTP Macros

A procedural macro library for the Nexus HTTP framework. This crate provides
high-level attributes to automate the boilerplate required for WebAssembly (WASI) and Fermyon Spin
service entry points, ensuring standardized telemetry, error handling, and security.

## Features

* **Runtime Abstraction:** A unified interface for both spin-sdk and standard WASI-HTTP components
  (via wit-bindgen).
* **Automatic Telemetry:** Generates tracing spans automatically, populating attributes like
  `http.method`, `http.route`, and `http.status_code`.
* **Distributed Tracing:** Seamlessly extracts W3C traceparent headers to maintain trace continuity
  and injects `x-trace-id` into outgoing responses.
* **Integrated Error Handling:** Intercepts `Result<T, E>` and transforms errors into structured
  JSON responses via the `ErrorMetadata` trait.

## Supported Macros

| Macro                    | Runtime              | Type  | Purpose                                               |
|--------------------------|----------------------|-------|-------------------------------------------------------|
| #[spin_service]          | Spin SDK             | async | Standard Spin services.                               |
| #[wasi_service]          | WASI Component Model | sync  | High-performance WASI-HTTP components.                |
| router!                  | Any                  | DSL   | Declarative route dispatching based on path segments. |

## Usage

### Spin Service (Asynchronous)

For services requiring the full spin-sdk asynchronous stack.

```rust,ignore
#[nx_http::error::error]
pub enum MyError {
    #[error(source = http::Error, code = 400, kind = "BAD_GATEWAY")]
    Http,
}

#[nx_http::spin_service]
async fn handle_request(req: Request) -> Result<impl IntoResponse, MyError> {
    // Asynchronous access to DB or Redis
    let data = fetch_from_kv().await?;
    Ok(data.into_response())
}
```

## WASI-HTTP Service (Synchronous)

Optimized for high-performance components built with `cargo-component`.

```rust,ignore
#[nx_http::error::error]
pub enum MyError {
    #[error(source = http::Error, code = 400, kind = "BAD_GATEWAY")]
    Http,
}

#[nx_http::wasi_service]
fn handle_request(request: Request<Bytes>) -> Result<Response<Bytes>, MyError> {
    // Clean, synchronous business logic
    Ok(Response::new(200, Bytes::from("Hello WASI Component!")))
}
```

## How it Works

The macros perform a code transformation during compilation:

1. **Context Extraction:** The macro generates code to look for the traceparent header. If found, it
   links the current execution to the parent trace.
2. **Telemetry Setup:** It creates an `info_span!` with metadata like the HTTP method and URI.
3. **Error Interception:** The handler is wrapped in a match or Result handler. If an error is
   returned, it is passed to the `nx-error` builder to return a valid JSON response to the client
   instead of crashing the Wasm instance.
4. **Tracing:** The `trace_id` is passed to `x-trace-id` response header to maintain trace
   continuity.

## Routing DSL (router!)

The `router!` macro provides a clean, pattern-matching based interface for dispatching requests within your services. It
is designed to be Zero-Copy, matching directly against the request path segments.

**Usage Example**

```rust,ignore
#[nx_http::spin_service]
async fn handle_request(req: Request, state: AppState) -> Result<Response, MyError> {
    nx_http::router!(
        req,
        // Static route
        get  ["health"] => Ok(Response::new(200, "OK")),
        
        // Captured segment (id: &str)
        get  ["user", id] => get_user_handler(id, &state),
        
        // Tail matching (rest: &[&str])
        get  ["static", ..rest] => serve_assets(rest),
        
        // Match with multiple dependencies
        post ["login"] => auth::login(&req, &state),
        
        // Fallback using your custom error macro
        _ => Err(MyError::not_found())
    )
}
```

## Requirements

* `nx-http`: This crate is a companion to `nx-http`.
* `wit-bindgen-rt` crate for clear WASI support.
* `cargo components` for generating the `wit-bindgen` bindings `bindings.rs`.

## License

Copyright © 2026 Nexus Project. All rights reserved.
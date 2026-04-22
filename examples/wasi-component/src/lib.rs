//! # Nexus WASI HTTP Component Example
//!
//! This crate serves as the reference implementation (Golden Path) for building
//! WASI HTTP components using the `nx-http` and `nx-error` stack.
//!
//! ## Core Features:
//! - **WIT Abstraction**: Complete isolation from low-level `wasi:http` guest bindings.
//! - **Type-Safe Handling**: Efficient conversion of WASI streams into standard `http::Request<Bytes>`.
//! - **Observability**: Automatic W3C Traceparent extraction and span propagation.
//! - **Error Contract**: Guaranteed serialization of domain errors into a machine-readable JSON format.

#[allow(clippy::all)]
#[allow(warnings)]
#[allow(unused_imports)]
#[allow(dead_code)]
pub mod bindings;
mod error;

use crate::error::WasiComponentError;
use nx_http::bytes::Bytes;
use nx_http::http::{Request, Response};

/// Primary business logic for the WASI HTTP component.
///
/// The `#[wasi_service]` macro orchestrates the following lifecycle:
/// 1. **Binding Transformation**: Maps WIT `IncomingRequest` to a standard `http::Request`.
/// 2. **Telemetry Instrumentation**: Initializes a `tracing` span and injects the `trace_id`
///    into the response headers for distributed correlation.
/// 3. **Declarative Error Recovery**: Intercepts `WasiComponentError`, logs the underlying
///    `source`, and renders a structured JSON response to the host.
#[cfg_attr(target_arch = "wasm32", nx_http::wasi_service)]
fn handle_request(request: Request<Bytes>) -> Result<Response<Bytes>, WasiComponentError> {
    // 1. Inspect request metadata
    let _method = request.method();
    let path = request.uri().path();

    // 2. Demonstrate Error-Driven Development:
    // This branch simulates a failure that is automatically caught by the macro
    // and transformed into a 500 Internal Server Error with a JSON body.
    if path == "/error" {
        return Err(WasiComponentError::internal()
            .with_details("Synthetic test error triggered by route")
            .with_help("Change the request path to bypass this simulated failure"));
    }

    // 3. Construct a successful response:
    // Using `Bytes` ensures zero-copy-like efficiency within the Wasm memory space.
    let response_body = Bytes::from("Hello from Nexus WASI Component!");

    Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(response_body)
        .map_err(|e| WasiComponentError::internal().with_details(e.to_string()))
}

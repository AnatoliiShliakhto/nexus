//! # Nexus Spin HTTP Service Example
//!
//! This component demonstrates the standardized "Golden Path" for building
//! observable and robust Wasm microservices using the `nx-http` toolkit.
//!
//! ## Core Capabilities
//! - **Asynchronous I/O**: Leverages `wasm32-wasip2` non-blocking HTTP via Spin SDK.
//! - **Distributed Tracing**: Automatic W3C `traceparent` propagation and span initialization.
//! - **Standardized Error Handling**: Automatic transformation of `Result::Err` into
//!   structured JSON responses following the Nexus error schema.
//! - **Type-Safe Proxying**: Fluent API for request forwarding with built-in header scrubbing.

mod error;

use crate::error::SpinComponentError;
use nx_http::spin_sdk::http::{IntoResponse, Request, send};
use nx_http::spin_tools::environment::get_spin_var;
use nx_http::spin_tools::proxy::ProxyRequestExt;

/// Entry point for the Spin HTTP component.
///
/// The `#[spin_service]` macro performs the following high-level operations:
/// 1. **Telemetry**: Initializes the tracing subscriber and extracts incoming trace context.
/// 2. **Execution**: Wraps the handler in a future and awaits the result.
/// 3. **Error Recovery**: If an error occurs, it intercepts the `SpinComponentError` and
///    renders a consistent JSON error body with appropriate HTTP status codes.
#[cfg_attr(target_arch = "wasm32", nx_http::spin_service)]
async fn handle_request(request: Request) -> Result<impl IntoResponse, SpinComponentError> {
    // Retrieve the target backend URL from Spin configuration variables.
    // Defaults to internal service mesh address if the variable is not explicitly set.
    let wasi_component_url =
        get_spin_var("nx_wasi_component_url", Some("http://nx-wasi-component.spin.internal"))?;

    // Construct an outbound proxy request using the fluent Builder API.
    // This automatically handles Hop-by-Hop header removal and path rebasing.
    let next = request.proxy_to(wasi_component_url).build()?;

    // Dispatch the request to the upstream service via the WASI-HTTP outgoing-handler.
    // Errors during transmission are automatically mapped to `SpinComponentError`.
    send(next).await.map(IntoResponse::into_response).map_err(SpinComponentError::from)
}

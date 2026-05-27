//! # Nexus HTTP Macros
//!
//! This crate provides procedural attributes to simplify the development of HTTP services
//! within the Spin and WASI Component Model ecosystems.

pub(crate) mod helpers;
mod spin;
mod wasi;

use crate::spin::RouterInput;
use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

/// Marks a function as a Spin HTTP service entry point.
///
/// This macro wraps an asynchronous handler to integrate it into the `nx-http` telemetry
/// and error handling pipeline, ensuring compatibility with the Spin WASI-HTTP runtime.
///
/// ### Transformations & Features:
/// 1. **Runtime Bridge**: Automatically generates the `wasi:http/incoming-handler`
///    guest implementation required by the Spin Wasm component model.
/// 2. **Observability (Telemetry)**:
///    - Initializes the tracing subscriber once per Wasm instance.
///    - Injects a root `http_request` tracing span containing `http.method`, `http.route`,
///      `traceid`, and `http.status_code` (recorded at runtime).
///    - Automatically extracts `traceparent` for distributed tracing and injects
///      `x-trace-id` into outgoing responses.
/// 3. **Standardized Error Handling**:
///    - Traps `Result<T, E>` where `E` implements `ErrorMetadata`.
///    - Automatically converts `Err` variants into structured JSON responses
///      (HTTP status code and internal error trace).
///
/// ### Usage
/// ```rust,ignore
/// #[nx_http::spin_service]
/// async fn handle_request(request: Request) -> Result<impl IntoResponse, MyNxError> {
///     // Logic implementation...
///     Ok(Response::builder().status(200).body("Hello Spin!").build())
/// }
/// ```
#[proc_macro_attribute]
pub fn spin_service(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    spin::derive_handler(&input_fn).into()
}

/// Marks a function as a WASI-HTTP component entry point.
///
/// This macro provides a seamless bridge between raw WASI-HTTP WIT exports and high-level
/// Rust logic. It is optimized for the WASI Component Model, enabling synchronous
/// handler execution while ensuring safety and observability.
///
/// ### Core Capabilities:
/// 1. **WASI Runtime Bridge**:
///    - Implements the `wasi:http/incoming-handler` guest interface via `wit-bindgen`.
///    - Manages the lifecycle of `IncomingRequest` and `ResponseOutparam` handles.
///
/// 2. **Observability & Telemetry**:
///    - Initializes the `tracing` subscriber once per Wasm instance lifetime.
///    - Instruments the handler with an `http_request` span, automatically tracking
///      `http.method`, `http.route`, `trace_id`, and `http.status_code`.
///    - Performs distributed trace propagation by extracting `traceparent` from
///      incoming headers and injecting `x-trace-id` into responses.
///
/// 3. **Resilient Error & Panic Handling**:
///    - Intercepts `Result<T, E>` and converts `Err` variants into structured JSON
///      responses, provided `E` implements `nx_http::error::ErrorMetadata`.
///
/// ### Requirements
/// - The error type `E` must implement `nx_http::error::ErrorMetadata`.
/// - The handler signature must be: `fn(Request<Bytes>) -> Result<Response<Bytes>, E>`.
/// - The crate must be configured with `cargo-component` bindings.
///
/// ### Example
/// ```rust,ignore
/// #[nx_http::wasi_service]
/// fn handle_request(request: Request<Bytes>) -> Result<Response<Bytes>, MyNxError> {
///     // Logic remains clean and synchronous
///     Ok(Response::new(200, Bytes::from("Hello WASI Component!")))
/// }
/// ```
#[proc_macro_attribute]
pub fn wasi_service(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    wasi::derive_service(&input_fn).into()
}

/// A routing DSL for `spin` components.
///
/// The `router!` macro provides a declarative way to map HTTP methods and path segments
/// directly to handler functions using Rust's native **Slice Pattern Matching**.
///
/// # Syntax
/// ```rust,ignore
/// router!(
///     request_instance,
///     method [path_segments] => handler_expr,
///     _ => fallback_expr
/// );
/// ```
///
/// # Examples
///
/// ### Basic Routing
/// ```rust,ignore
/// router!(
///     request,
///     get  ["health"] => Ok(Response::new(200, "OK")),
///     post ["login"]  => auth::handle_login(&request, &state),
///     _ => Err(AppError::not_found())
/// );
/// ```
///
/// ### Dynamic Path Parameters
/// You can capture segments directly into variables. Captured variables are `&str`.
/// ```rust,ignore
/// router!(
///     request,
///     // Captures the second segment as 'id'
///     get ["user", id] => get_user_handler(id, &database),
///     _ => Err(AppError::not_found())
/// );
/// ```
///
/// ### Tail Matching (Wildcards)
/// Capture the remainder of a path as a slice `&[&str]`.
/// ```rust,ignore
/// router!(
///     request,
///     // Matches /static/css/main.css -> tail is ["css", "main.css"]
///     get ["static", ..tail] => serve_static_assets(tail),
///     _ => Err(AppError::not_found())
/// );
/// ```
#[proc_macro]
pub fn router(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as RouterInput);
    spin::router_derive(&input).into()
}

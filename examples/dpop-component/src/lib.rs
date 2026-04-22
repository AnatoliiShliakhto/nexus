//! # `DPoP` Service Dispatcher
//!
//! This module serves as the primary HTTP entry point for the `nx-dpop` service.
//! It handles request routing, semantic validation, and payload deserialization
//! before delegating to the internal cryptographic business logic.
//!
//! ## Request Handling Flow
//!
//! The handler enforces strict constraints to ensure system stability and security:
//! * **Method:** Only `POST` requests are permitted.
//! * **Content-Type:** Only `application/octet-stream` is accepted.
//! * **Payload Size:** Requests are capped at 32 KB to prevent OOM errors.

#![cfg_attr(
    not(target_arch = "wasm32"),
    allow(dead_code, unused_imports, clippy::needless_for_each)
)]

use crate::dpop::{handle_compute_thumbprint, handle_verify_proof};
use crate::error::DpopError;
use nx_http::router;
use nx_http::spin_sdk::http::{IntoResponse, Request};

mod dpop;
mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod openapi;

/// The primary service entry point.
///
/// Dispatches incoming HTTP requests to either the thumbprint computation
/// engine or the `DPoP` proof verification engine based on the request path.
///
/// # Errors
/// Returns `DpopError` if:
/// * The HTTP method is not `POST`.
/// * The `Content-Type` is not `application/octet-stream`.
/// * The request body exceeds the 32 KB limit.
/// * The request path is invalid.
/// * Deserialization of the binary payload fails.
#[cfg_attr(target_arch = "wasm32", nx_http::spin_service)]
async fn handle_request(request: Request) -> Result<impl IntoResponse, DpopError> {
    validate_request(&request)?;

    router!(request,
        Post ["compute"] => handle_compute_thumbprint(request).await,
        Post ["verify"] => handle_verify_proof(request).await,
        _ => Err(DpopError::not_found()),
    )
}

fn validate_request(request: &Request) -> Result<(), DpopError> {
    if request.header("content-type").and_then(|v| v.as_str()) != Some("application/octet-stream") {
        return Err(DpopError::unsupported_content());
    }

    if request.body().len() > 32768 {
        return Err(DpopError::bad_request().with_details("Proof too large"));
    }

    Ok(())
}

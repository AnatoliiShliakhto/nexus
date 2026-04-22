#![cfg_attr(
    not(target_arch = "wasm32"),
    allow(dead_code, unused_imports, clippy::needless_for_each)
)]

mod error;

use crate::error::OrganizationError;
use nx_http::spin_sdk::http::{IntoResponse, Request};
#[cfg(not(target_arch = "wasm32"))]
pub mod openapi;

#[cfg_attr(target_arch = "wasm32", nx_http::spin_service)]
async fn handle_request(_request: Request) -> Result<impl IntoResponse, OrganizationError> {
    Ok(())
}

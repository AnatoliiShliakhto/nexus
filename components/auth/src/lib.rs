#![cfg_attr(
    not(target_arch = "wasm32"),
    allow(dead_code, unused_imports, clippy::needless_for_each)
)]

mod error;

use crate::error::AuthError;
use nx_http::router;
use nx_http::spin_sdk::http::{IntoResponse, Request};
mod handlers;
#[cfg(not(target_arch = "wasm32"))]
pub mod openapi;

#[cfg_attr(target_arch = "wasm32", nx_http::spin_service)]
async fn handle_request(request: Request) -> Result<impl IntoResponse, AuthError> {
    router!(request,
        Post   ["login"] => handlers::handle_login(&request).await,
        Get    ["token", "refresh"] => handlers::handle_token_refresh(&request).await,
        Delete ["token", "revoke"] => handlers::handle_token_revoke(&request).await,
        _ => Err(AuthError::not_found()),
    )
}

#![cfg_attr(
    not(target_arch = "wasm32"),
    allow(dead_code, unused_imports, clippy::needless_for_each)
)]

mod error;

use crate::error::{{ shortname | pascal_case }}Error;
use nx_http::spin_sdk::http::{IntoResponse, Request, Response};
#[cfg(not(target_arch = "wasm32"))]
pub mod openapi;

#[cfg_attr(target_arch = "wasm32", nx_http::spin_service)]
async fn handle(_request: Request) -> Result<impl IntoResponse, {{ shortname | pascal_case }}Error> {

    Ok(Response::new(200, ()))
}

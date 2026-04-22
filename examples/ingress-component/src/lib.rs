#![cfg_attr(
    not(target_arch = "wasm32"),
    allow(dead_code, unused_imports, clippy::needless_for_each)
)]

mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod openapi;

use crate::error::IngressError;
use nx_http::spin_sdk::http::{IntoResponse, Request, send};
use nx_http::spin_tools::environment::get_spin_var;
use nx_http::spin_tools::proxy::ProxyRequestExt;

const DEFAULT_GATEWAY_URL: &str = "http://nx-gateway-component.spin.internal";

#[cfg_attr(target_arch = "wasm32", nx_http::spin_service)]
async fn handle_response(request: Request) -> Result<impl IntoResponse, IngressError> {
    if request.uri().path().starts_with("/api/health") {
        return Ok("OK".into_response());
    }

    let gateway_url = get_spin_var("nx_gateway_component_url", Some(DEFAULT_GATEWAY_URL)).await?;
    let next = request.proxy_to(gateway_url).build()?;

    send(next).await.map(IntoResponse::into_response).map_err(IngressError::from)
}

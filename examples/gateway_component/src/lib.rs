mod error;
mod router;

use crate::error::GatewayError;
use crate::router::Route;
use nx_http::spin_sdk::http::{IntoResponse, Request, send};
use nx_http::spin_tools::environment::get_spin_var;
use nx_http::spin_tools::proxy::ProxyRequestExt;

#[cfg_attr(target_arch = "wasm32", nx_http::spin_service)]
async fn handle_request(request: Request) -> Result<impl IntoResponse, GatewayError> {
    let route = Route::from_request(&request)?;
    let meta = route.metadata();

    let url = get_spin_var(meta.config_key, meta.default_url)?;
    let next = request.proxy_to(url).strip_prefix(meta.prefix).build()?;

    send(next).await.map(IntoResponse::into_response).map_err(GatewayError::from)
}

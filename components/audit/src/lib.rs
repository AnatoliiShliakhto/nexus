mod error;

use crate::error::AuditError;
use nx_http::spin_sdk::http::{IntoResponse, Request};

#[cfg_attr(target_arch = "wasm32", nx_http::spin_service)]
async fn handle_request(_request: Request) -> Result<impl IntoResponse, AuditError> {
    Ok(())
}

use crate::error::AuthError;
use nx_http::spin_sdk::http::{IntoResponse, Request};

pub(crate) async fn handle_token_refresh(
    _request: &Request,
) -> Result<impl IntoResponse, AuthError> {
    Ok(())
}

pub(crate) async fn handle_token_revoke(
    _request: &Request,
) -> Result<impl IntoResponse, AuthError> {
    Ok(())
}

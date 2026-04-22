use crate::error::AuthError;
use nx_http::spin_sdk::http::{IntoResponse, Request};
use serde::Deserialize;
use std::fmt::{Debug, Formatter};

#[cfg_attr(not(target_arch = "wasm32"), derive(utoipa::ToSchema), schema(as = AuthLoginRequest))]
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

impl Debug for LoginRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginRequest")
            .field("username", &self.username)
            .field("password", &"********")
            .finish()
    }
}

pub(crate) async fn handle_login(_request: &Request) -> Result<impl IntoResponse, AuthError> {
    Ok("Login successful!".to_owned())
}

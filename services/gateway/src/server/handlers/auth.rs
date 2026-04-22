use crate::error::GatewayError;
use crate::features::identity::error::IdentityError;
use crate::features::identity::session::models::Actions;
use crate::server::extractors::identity::IdentityContext;
use crate::server::state::GatewayState;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use fxhash::FxHashMap;
use http::header;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Deserialize)]
pub(crate) struct AuthorizationRequest {
    login: String,
    password: String,
}

impl Debug for AuthorizationRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthorizationRequest")
            .field("login", &self.login)
            .field("password", &"********")
            .finish()
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct AuthorizationResponse {
    token: String,
    permissions: FxHashMap<String, Actions>,
}

pub(crate) async fn authorization_handler(
    State(state): State<GatewayState>,
    ident: IdentityContext,
    Json(payload): Json<AuthorizationRequest>,
) -> Result<impl IntoResponse, GatewayError> {
    let AuthorizationRequest { login, password } = payload;
    let (refresh_token, session) =
        state.sessions.authenticate_with_credentials(&login, &password, &ident).await?;

    let response = AuthorizationResponse {
        token: session.id.to_string(),
        permissions: session.permissions.clone(),
    };

    Ok((
        [(
            header::SET_COOKIE,
            format!(
                "rt={refresh_token}; Path=/api/auth/tokens/refresh; HttpOnly; Secure; SameSite=Lax"
            ),
        )],
        Json(response),
    ))
}

pub(crate) async fn token_refresh_handler(
    State(state): State<GatewayState>,
    ident: IdentityContext,
    jar: CookieJar,
) -> Result<impl IntoResponse, GatewayError> {
    let rt = jar
        .get("rt")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| IdentityError::refresh_token_missing())?;

    let (refresh_token, session) = state.sessions.refresh_session(&rt, &ident).await?;

    let response = AuthorizationResponse {
        token: session.id.to_string(),
        permissions: session.permissions.clone(),
    };

    Ok((
        [(
            header::SET_COOKIE,
            format!(
                "rt={refresh_token}; Path=/api/auth/tokens/refresh; HttpOnly; Secure; SameSite=Lax"
            ),
        )],
        Json(response),
    ))
}

pub(crate) async fn token_revoke_handler(
    State(state): State<GatewayState>,
    ident: IdentityContext,
) -> Result<impl IntoResponse, GatewayError> {
    let _ = state.sessions.revoke_session(&ident).await;

    Ok((
        [(
            header::SET_COOKIE,
            "rt=; Path=/api/auth/tokens/refresh; HttpOnly; Secure; SameSite=Lax; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
        )],
        http::StatusCode::NO_CONTENT,
    ))
}

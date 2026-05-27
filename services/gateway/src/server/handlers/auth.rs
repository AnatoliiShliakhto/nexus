use crate::error::GatewayError;
use crate::features::identity::error::IdentityError;
use crate::features::identity::session::models::{Actions, SessionStatus};
use crate::server::extractors::identity::IdentityContext;
use crate::server::state::GatewayState;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::{CookieJar, WithRejection};
use fxhash::FxHashMap;
use http::{HeaderValue, header};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fmt::{Debug, Formatter};

static REVOKE_TOKEN_HEADER_VALUE: HeaderValue = HeaderValue::from_static(
    "rt=; Path=/api/auth/tokens/refresh; HttpOnly; Secure; SameSite=Lax; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
);

#[derive(Deserialize)]
pub(crate) struct AuthorizationRequest {
    username: String,
    password: String,
}

impl Debug for AuthorizationRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthorizationRequest")
            .field("username", &self.username)
            .field("password", &"********")
            .finish()
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RefreshRequest {
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AuthorizationResponse {
    status: SessionStatus,
    access_token: String,
    access_token_expires_at: u64,
    refresh_token: String,
    permissions: FxHashMap<String, Actions>,
}

pub(crate) async fn authorization_handler(
    State(state): State<GatewayState>,
    ident: IdentityContext,
    WithRejection(Json(payload), _): WithRejection<Json<AuthorizationRequest>, GatewayError>,
) -> Result<impl IntoResponse, GatewayError> {
    let access_token_expires_at = chrono::Utc::now().timestamp().cast_unsigned()
        + state.config.get().security.identity.session.access_token_ttl_sec;
    let AuthorizationRequest { username, password } = payload;

    let (refresh_token, session) =
        state.sessions.authenticate_with_credentials(&username, &password, &ident).await?;

    let response = AuthorizationResponse {
        status: session.status,
        access_token: session.id.to_string(),
        access_token_expires_at,
        refresh_token: refresh_token.clone(),
        permissions: session.permissions.clone(),
    };

    Ok(([(header::SET_COOKIE, generate_rt_header_value(&refresh_token))], Json(response)))
}

pub(crate) async fn token_refresh_handler(
    State(state): State<GatewayState>,
    ident: IdentityContext,
    jar: CookieJar,
    payload: Option<Json<RefreshRequest>>,
) -> Result<impl IntoResponse, GatewayError> {
    let rt = payload
        .and_then(|Json(p)| p.refresh_token)
        .or_else(|| jar.get("rt").map(|cookie| cookie.value().to_string()))
        .ok_or_else(|| IdentityError::refresh_token_missing())?;
    let access_token_expires_at = chrono::Utc::now().timestamp().cast_unsigned()
        + state.config.get().security.identity.session.access_token_ttl_sec;

    let (refresh_token, session) = state.sessions.refresh_session(&rt, &ident).await?;

    let response = AuthorizationResponse {
        status: session.status,
        access_token: session.id.to_string(),
        access_token_expires_at,
        refresh_token: refresh_token.clone(),
        permissions: session.permissions.clone(),
    };

    Ok(([(header::SET_COOKIE, generate_rt_header_value(&refresh_token))], Json(response)))
}

pub(crate) async fn token_revoke_handler(
    State(state): State<GatewayState>,
    ident: IdentityContext,
) -> Result<impl IntoResponse, GatewayError> {
    let _ = state.sessions.validate_session(&ident).await?;
    state.sessions.revoke_session(&ident).await?;

    Ok(([(header::SET_COOKIE, REVOKE_TOKEN_HEADER_VALUE.clone())], http::StatusCode::NO_CONTENT))
}

// --- Helpers ---

fn generate_rt_header_value(token: &str) -> HeaderValue {
    let mut buf = SmallVec::<u8, 256>::new();

    buf.extend_from_slice(b"rt=");
    buf.extend_from_slice(token.as_bytes());
    buf.extend_from_slice(b"; Path=/api/auth/tokens/refresh; HttpOnly; Secure; SameSite=Lax");

    HeaderValue::try_from(&*buf).unwrap_or_else(|_| HeaderValue::from_static(""))
}

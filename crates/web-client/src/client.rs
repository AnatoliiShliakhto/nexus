//! Web Client implementation for the NEXUS ecosystem.
//!
//! This module provides [`WebClient`], a thread-safe, asynchronous HTTP client
//! specialized for Dioxus frontends with built-in DPoP authentication and session tracking.

use crate::dpop::DpopSigner;
use crate::error::{ApiErrorBody, WebClientError};
use crate::session::{Actions, Session, SessionStatus};
use crate::{IntoBody, Json, RequestBody, ResponseExt};
use arc_swap::ArcSwapOption;
use fxhash::FxHashMap;
use nx_error::ErrorMetadata;
use nx_lockbox::Lockbox;
use reqwest::{
    Client, ClientBuilder, Method, Request, Url,
    header::{self, HeaderValue},
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

/// The primary entry point for interacting with NEXUS services.
///
/// `WebClient` handles:
/// - **DPoP Authentication**: Automatic proof generation for every request.
/// - **Token Lifecycle**: Proactive and reactive refresh of access tokens.
/// - **Session Persistence**: Secure storage of refresh tokens via [`Lockbox`].
/// - **Reactivity**: Provides a `watch` channel for session state changes.
#[derive(Debug, Clone)]
pub struct WebClient {
    inner: Arc<WebClientInner>,
}

#[derive(Debug)]
struct WebClientInner {
    client: Client,
    base_url: Url,
    signer: DpopSigner,
    access_token: ArcSwapOption<AccessToken>,
    refresh_token: ArcSwapOption<String>,
    lockbox: Lockbox,
    refresh_in_flight: tokio::sync::Mutex<()>,
    session_tx: tokio::sync::watch::Sender<Arc<Session>>,
    session_rx: tokio::sync::watch::Receiver<Arc<Session>>,
}

/// A thread-safe container for the active access token and its metadata.
#[derive(Debug)]
struct AccessToken {
    /// The raw JWT string.
    value: String,
    /// Pre-computed `Authorization` header value to avoid repeated allocations.
    authorization_header: HeaderValue,
    /// Expiration timestamp stored as an atomic for lock-free checks.
    expires_at: AtomicU64,
}

impl AccessToken {
    /// Creates a new `AccessToken` and pre-computes the DPoP Authorization header.
    ///
    /// # Errors
    /// Returns an error if the token contains characters invalid for HTTP headers.
    fn try_new(value: String, expires_at: u64) -> Result<Self, WebClientError> {
        let authorization_header = HeaderValue::from_str(&format!("DPoP {value}"))?;
        Ok(Self { value, authorization_header, expires_at: AtomicU64::new(expires_at) })
    }

    /// Returns the expiration timestamp.
    fn expires_at(&self) -> u64 {
        self.expires_at.load(Ordering::Relaxed)
    }
}

impl WebClient {
    /// Initializes a new `WebClient`.
    ///
    /// # Arguments
    /// * `service_name` - Unique name for the service (used for secure storage isolation).
    /// * `base_url` - The root URL of the API gateway.
    ///
    /// # Errors
    /// Returns [`WebClientError`] if the HTTP client or DPoP signer fails to initialize.
    pub fn init(
        service_name: impl Into<Cow<'static, str>>,
        base_url: impl Into<Url>,
    ) -> Result<Self, WebClientError> {
        let user_agent = format!("Nexus Web Client/{}", env!("CARGO_PKG_VERSION"));
        let client = ClientBuilder::new().user_agent(user_agent).cookie_store(true).build()?;
        let signer = DpopSigner::init()?;
        let lockbox = Lockbox::new(service_name.into())?;
        let refresh_token = ArcSwapOption::from_pointee(lockbox.get("rt").ok());
        let session = lockbox.get_json::<Session>("session").unwrap_or_default();
        let (session_tx, session_rx) = tokio::sync::watch::channel(Arc::new(session));

        let inner = WebClientInner {
            client,
            base_url: base_url.into(),
            signer,
            access_token: ArcSwapOption::empty(),
            refresh_token,
            lockbox,
            refresh_in_flight: tokio::sync::Mutex::new(()),
            session_tx,
            session_rx,
        };

        Ok(Self { inner: Arc::new(inner) })
    }

    /// Performs an authorized GET request.
    pub async fn get(&self, path: &str) -> Result<reqwest::Response, WebClientError> {
        self.request(Method::GET, path, RequestBody::Empty).await
    }

    /// Performs an authorized POST request with a serializable body.
    pub async fn post<B: IntoBody>(
        &self,
        path: &str,
        body: B,
    ) -> Result<reqwest::Response, WebClientError> {
        self.request(Method::POST, path, body.into_body()).await
    }

    /// Performs an authorized PUT request with a serializable body.
    pub async fn put<B: IntoBody>(
        &self,
        path: &str,
        body: B,
    ) -> Result<reqwest::Response, WebClientError> {
        self.request(Method::PUT, path, body.into_body()).await
    }

    /// Performs an authorized DELETE request with a serializable body.
    pub async fn delete(&self, path: &str) -> Result<reqwest::Response, WebClientError> {
        self.request(Method::DELETE, path, RequestBody::Empty).await
    }

    pub async fn health(&self) -> Result<(), WebClientError> {
        self.get("/api/health").await?;
        Ok(())
    }

    /// Authenticates a user with a username and password.
    ///
    /// On success, tokens are stored securely, and the session state is updated.
    pub async fn login_with_credentials(
        &self,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Result<(), WebClientError> {
        let payload = AuthorizationRequestWithCredentials {
            username: username.as_ref(),
            password: password.as_ref(),
        };

        let response = self
            .send_without_refresh(Method::POST, "/api/auth/tokens", Json(payload).into_body(), None)
            .await?;

        let authorization = response.resolve::<AuthorizationResponse>().await?;
        self.set_authorized_session(authorization)?;

        tracing::info!(username = %username.as_ref(), "User logged in successfully");

        Ok(())
    }

    /// Explicitly triggers an access token refresh using the stored refresh token.
    pub async fn refresh_token(&self) -> Result<(), WebClientError> {
        self.refresh_token_once().await
    }

    /// Logs out the user and clears all local session data.
    pub async fn logout(&self) {
        let token_snapshot = self.current_access_token();

        if let Err(e) = self
            .send_without_refresh(
                Method::DELETE,
                "/api/auth/tokens",
                RequestBody::Empty,
                token_snapshot,
            )
            .await
        {
            tracing::error!(error = %e.message(), code = %e.code(), details = %e.details().unwrap_or_default(), "Failed to logout user");
        } else {
            tracing::info!("User logged out successfully");
        }

        self.clear_session();
    }

    /// Returns a reactive subscriber for session state changes.
    ///
    /// Ideal for use in Dioxus `use_coroutine` or `use_resource` to update UI
    /// based on login/logout events.
    pub fn session_rx(&self) -> tokio::sync::watch::Receiver<Arc<Session>> {
        self.inner.session_rx.clone()
    }

    async fn handle_response(
        &self,
        response: reqwest::Response,
    ) -> Result<reqwest::Response, WebClientError> {
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

        let path = response.url().path().to_owned();
        let bytes = response.bytes().await.map_err(WebClientError::from)?;

        if let Ok(error) = serde_json::from_slice::<ApiErrorBody>(&bytes) {
            if let Some(Ok(details)) = error.details.map(|value| serde_json::to_string(&value)) {
                tracing::error!(
                    target: "nx_web_client::api",
                    status = %status,
                    code = %error.code,
                    path = %path,
                    message = %error.message,
                    details = %details,
                    "API Request Failed"
                );
            }

            return Err(WebClientError::api_error()
                .with_status(error.status)
                .with_code(error.code)
                .with_message(error.message));
        }

        let fallback = String::from_utf8_lossy(&bytes).into_owned();

        tracing::warn!(
            status = %status,
            body = %fallback,
            path = %path,
            "Unexpected Server Response"
        );

        Err(WebClientError::api_error().with_status(status.as_u16()).with_details(fallback))
    }

    /// Core request orchestration logic.
    ///
    /// Handles pre-request token freshness checks and automatic retries on 401 Unauthorized.
    async fn request(
        &self,
        method: Method,
        path: &str,
        body: RequestBody,
    ) -> Result<reqwest::Response, WebClientError> {
        if self.should_refresh_before_request() {
            self.ensure_fresh_token().await?;
        }

        let response =
            self.send_once(method.clone(), path, body.clone(), self.current_access_token()).await?;

        if response.status() != reqwest::StatusCode::UNAUTHORIZED {
            return self.handle_response(response).await;
        }

        if self.current_access_token().is_none() {
            return self.handle_response(response).await;
        }

        self.ensure_fresh_token().await?;

        let retry_response =
            self.send_once(method, path, body, self.current_access_token()).await?;

        self.handle_response(retry_response).await
    }

    async fn send_without_refresh(
        &self,
        method: Method,
        path: &str,
        body: RequestBody,
        token: Option<Arc<AccessToken>>,
    ) -> Result<reqwest::Response, WebClientError> {
        let response = self.send_once(method, path, body, token).await?;
        self.handle_response(response).await
    }

    async fn send_once(
        &self,
        method: Method,
        path: &str,
        body: RequestBody,
        token: Option<Arc<AccessToken>>,
    ) -> Result<reqwest::Response, WebClientError> {
        let url = self
            .inner
            .base_url
            .join(path)
            .map_err(|error| WebClientError::internal().with_details(error.to_string()))?;

        let mut builder = self.inner.client.request(method, url);

        #[cfg(target_arch = "wasm32")]
        {
            builder = builder.credentials(reqwest::wasm::Credentials::Include);
        }

        builder = match body {
            RequestBody::Json(value) => builder.json(&value),
            RequestBody::Bytes(content_type, bytes) => {
                builder.body(bytes).header(header::CONTENT_TYPE, content_type)
            },
            RequestBody::Form(form) => builder.form(&form),
            RequestBody::Empty => builder,
            RequestBody::Multipart(multipart) => {
                let mut form = reqwest::multipart::Form::new();

                for (name, value) in multipart.fields {
                    form = form.text(name, value);
                }

                for (field, (filename, data)) in multipart.files {
                    let part = reqwest::multipart::Part::stream(data).file_name(filename);
                    form = form.part(field, part);
                }

                builder.multipart(form)
            },
        };

        let mut request = builder.build().map_err(WebClientError::from)?;
        self.apply_auth(&mut request, token.as_deref())?;

        self.inner.client.execute(request).await.map_err(WebClientError::from)
    }

    /// Applies DPoP and Authorization headers to a raw request.
    fn apply_auth(
        &self,
        request: &mut Request,
        token: Option<&AccessToken>,
    ) -> Result<(), WebClientError> {
        let mut htu = request.url().clone();
        htu.set_query(None);
        htu.set_fragment(None);

        let proof = self.inner.signer.prove(
            request.method().as_str(),
            htu.as_str(),
            token.map(|token| token.value.as_str()),
        )?;

        request.headers_mut().insert("DPoP", HeaderValue::from_str(&proof)?);

        if let Some(token) = token {
            request.headers_mut().insert(header::AUTHORIZATION, token.authorization_header.clone());
        }

        Ok(())
    }

    async fn ensure_fresh_token(&self) -> Result<(), WebClientError> {
        let _guard = self.inner.refresh_in_flight.lock().await;

        if self.is_token_fresh() {
            return Ok(());
        }

        match self.refresh_token_once().await {
            Ok(()) => Ok(()),
            Err(e) => {
                if e.status().as_u16() == 400 || e.status().as_u16() == 401 {
                    tracing::warn!(
                        error = %e.message(),
                        status = %e.status().as_u16(),
                        code = %e.code(),
                        details = %e.details().unwrap_or_default(),
                        "Critical auth error during token refresh. Forcing logout."
                    );
                    self.clear_session();
                }
                Err(e)
            },
        }
    }

    async fn refresh_token_once(&self) -> Result<(), WebClientError> {
        tracing::debug!("Attempting to refresh access token");

        let Some(rt) = self.current_refresh_token() else {
            self.clear_session();
            return Ok(());
        };
        let payload = RefreshTokenRequest { refresh_token: rt.to_string() };
        let response = self
            .send_without_refresh(
                Method::POST,
                "/api/auth/tokens/refresh",
                Json(payload).into_body(),
                self.current_access_token(),
            )
            .await?;

        let authorization = response.resolve::<AuthorizationResponse>().await?;
        self.set_authorized_session(authorization)?;

        tracing::info!("Access token refreshed successfully");
        Ok(())
    }

    fn set_authorized_session(
        &self,
        authorization: AuthorizationResponse,
    ) -> Result<(), WebClientError> {
        let AuthorizationResponse {
            status,
            access_token,
            access_token_expires_at,
            refresh_token,
            permissions,
        } = authorization;

        if let Err(e) = self.inner.lockbox.set("rt", &refresh_token) {
            tracing::error!(error = %e, "Failed to store refresh token in lockbox");
        }
        let session = Session { status, permissions };
        if let Err(e) = self.inner.lockbox.set_json("session", &session) {
            tracing::error!(error = %e, "Failed to store session in lockbox");
        }

        let access_token = AccessToken::try_new(access_token, access_token_expires_at)?;
        self.inner.access_token.store(Some(Arc::new(access_token)));
        self.inner.refresh_token.store(Some(Arc::from(refresh_token)));

        self.inner.session_tx.send_replace(Arc::new(session));

        Ok(())
    }

    fn clear_session(&self) {
        if let Err(e) = self.inner.lockbox.delete("rt") {
            tracing::error!(error = %e, "Failed to delete refresh token from lockbox");
        }
        if let Err(e) = self.inner.lockbox.delete("session") {
            tracing::error!(error = %e, "Failed to delete session from lockbox");
        }
        self.inner.access_token.store(None);
        self.inner.refresh_token.store(None);
        self.inner.session_tx.send_replace(Arc::new(Session::default()));
    }

    fn current_access_token(&self) -> Option<Arc<AccessToken>> {
        self.inner.access_token.load_full()
    }

    #[allow(clippy::rc_buffer)]
    fn current_refresh_token(&self) -> Option<Arc<String>> {
        self.inner.refresh_token.load_full()
    }

    /// Determines if the current access token is about to expire.
    ///
    /// Returns `true` if the token expires in less than 10 seconds.
    fn is_token_fresh(&self) -> bool {
        let Some(token) = self.current_access_token() else {
            return false;
        };

        token.expires_at() > now().saturating_add(10)
    }

    /// Logical check to decide if a refresh flow should be triggered before the next request.
    fn should_refresh_before_request(&self) -> bool {
        let at = self.current_access_token();
        let rt = self.current_refresh_token();

        if at.is_none() && rt.is_some() {
            return true;
        }

        at.is_some_and(|token| token.expires_at() <= now().saturating_add(10))
    }
}

// --- Models ---

/// Represents the payload for a credential-based authorization request.
#[derive(Debug, Serialize)]
pub(crate) struct AuthorizationRequestWithCredentials<'a> {
    /// The unique username of the account.
    pub username: &'a str,
    /// The plain-text password (will be sent over TLS).
    pub password: &'a str,
}

/// The server's response to a successful authorization or refresh request.
#[derive(Debug, Deserialize)]
pub(crate) struct AuthorizationResponse {
    /// Status of the established session.
    pub status: SessionStatus,
    /// The DPoP-bound access token string.
    pub access_token: String,
    /// Unix timestamp (seconds) when the access token becomes invalid.
    pub access_token_expires_at: u64,
    /// Long-lived token used to get access tokens.
    pub refresh_token: String,
    /// Map of component identifiers to their respective granted [`Actions`].
    pub permissions: FxHashMap<String, Actions>,
}

#[derive(Debug, Serialize)]
struct RefreshTokenRequest {
    refresh_token: String,
}

// --- Helpers ---

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

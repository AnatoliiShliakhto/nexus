use super::request::ProxyRequestExt;
use crate::error::GatewayError;
use crate::infra::telemetry::METRICS;
use crate::server::extractors::identity::IdentityContext;
use crate::server::state::GatewayState;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    http::{Request, Response},
};
use crossbeam_epoch as epoch;
use futures_util::future::BoxFuture;
use http::{HeaderName, HeaderValue};
use http_body_util::BodyExt;
use smol_str::SmolStr;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};
use url::Url;

const X_SURREAL_TOKEN: HeaderName = HeaderName::from_static("x-surreal-token");
const X_SESSION_ID: HeaderName = HeaderName::from_static("x-session-id");

#[derive(Debug, Clone)]
pub(crate) struct ProxyFilterLayer {
    pub state: GatewayState,
}

impl ProxyFilterLayer {
    /// Creates a new `ProxyFilterLayer` with the shared Gateway state.
    /// Since `GatewayState` is designed to be inexpensive to clone (Arcs inside),
    /// we pass it by value.
    pub(crate) const fn new(state: GatewayState) -> Self {
        Self { state }
    }
}
impl<S> Layer<S> for ProxyFilterLayer {
    type Service = ProxyFilterService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ProxyFilterService { inner, state: self.state.clone() }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProxyFilterService<S> {
    inner: S,
    state: GatewayState,
}

impl<S> Service<Request<Body>> for ProxyFilterService<S>
where
    S: Service<Request<Body>, Response = Response<Body>, Error = std::convert::Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let state = self.state.clone();

        Box::pin(async move {
            let path = req.uri().path();

            let Some((component, target, protected)) = ({
                let guard = epoch::pin();
                state
                    .routes
                    .match_prefix(path, &guard)
                    .map(|route| (route.component.clone(), route.target.clone(), route.protected))
            }) else {
                return inner.call(req).await;
            };

            match proxy_request(state, component, target, protected, req).await {
                Ok(res) => Ok(res),
                Err(err) => Ok(err.into_response()),
            }
        })
    }
}

async fn proxy_request(
    state: GatewayState,
    component: SmolStr,
    target: Url,
    protected: bool,
    req: Request<Body>,
) -> Result<Response<Body>, GatewayError> {
    let start = Instant::now();

    let proxy_req = if protected {
        let ident = IdentityContext::from_request(&req);
        let session = state.sessions.validate_session(&ident).await?;
        if !session.check_access(&component, &req.method()) {
            return Err(GatewayError::access_denied()
                .with_details(format!("Lack of permissions for component: {component}")));
        }

        req.proxy_to(&target)
            .header(
                X_SESSION_ID,
                HeaderValue::try_from(session.id.as_str()).unwrap_or(HeaderValue::from_static("")),
            )
            .header(X_SURREAL_TOKEN, session.database_token.clone())
            .build()?
    } else {
        req.proxy_to(&target).build()?
    };

    let res = state.client.request(proxy_req).await;

    let response = res.map_err(|e| {
        METRICS.record_proxy_error("dispatch_failed");
        GatewayError::dispatch_failed().with_details(e.to_string())
    })?;

    METRICS.record_proxy_request(response.status().as_u16(), start.elapsed().as_secs_f64());

    let (parts, body) = response.into_parts();
    let body = Body::from_stream(body.into_data_stream());
    Ok(axum::response::Response::from_parts(parts, body))
}

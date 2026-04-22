use crate::infra::ip_value::IpValue;
use axum::extract::{ConnectInfo, FromRequestParts, Request};
use http::request::Parts;
use http::{HeaderMap, HeaderName, Method, Uri, header};
use std::net::SocketAddr;
use std::str::FromStr;
use smol_str::SmolStr;

pub(crate) const CF_CONNECTING_IP: HeaderName = HeaderName::from_static("cf-connecting-ip");
pub(crate) const X_FORWARDED_FOR: HeaderName = HeaderName::from_static("x-forwarded-for");
pub(crate) const X_REAL_IP: HeaderName = HeaderName::from_static("x-real-ip");
pub(crate) const DPOP: HeaderName = HeaderName::from_static("dpop");
pub(crate) const X_DPOP_NONCE: HeaderName = HeaderName::from_static("x-dpop-nonce");

#[derive(Debug)]
pub(crate) struct IdentityContext {
    pub session_id: Option<SmolStr>,
    pub dpop_proof: Option<Box<str>>,
    pub dpop_nonce: Option<SmolStr>,
    pub htu: SmolStr,
    pub htm: Method,
    pub ip: Option<IpValue>,
    pub user_agent: Option<SmolStr>,
}

impl IdentityContext {
    pub(crate) fn from_request<B>(req: &Request<B>) -> Self {
        Self::from_parts(req.method(), req.uri(), req.headers(), req.extensions())
    }

    pub(crate) fn from_parts(
        method: &Method,
        uri: &Uri,
        headers: &HeaderMap,
        extensions: &http::Extensions,
    ) -> Self {
        let ip = extract_ip(headers).or_else(|| {
            extensions.get::<ConnectInfo<SocketAddr>>().map(|addr| IpValue::from(addr.0.ip()))
        });

        let htu = SmolStr::new(uri.path());
        let htm = method.clone();

        let session_id = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("DPoP "))
            .or_else(|| {
                headers
                    .get(header::AUTHORIZATION)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| {
                        if s.get(..5).map_or(false, |p| p.eq_ignore_ascii_case("DPoP ")) {
                            Some(&s[5..])
                        } else {
                            None
                        }
                    })
            })
            .map(SmolStr::from);

        let dpop_proof = headers.get(&DPOP).and_then(|v| v.to_str().ok()).map(Box::from);

        let dpop_nonce = headers.get(&X_DPOP_NONCE).and_then(|v| v.to_str().ok()).map(SmolStr::from);

        let user_agent = headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()).map(SmolStr::from);

        Self { session_id, dpop_proof, dpop_nonce, htu, htm, ip, user_agent }
    }
}

impl<S> FromRequestParts<S> for IdentityContext
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_parts(&parts.method, &parts.uri, &parts.headers, &parts.extensions))
    }
}

// --- Helpers ---

#[inline]
fn extract_ip(headers: &HeaderMap) -> Option<IpValue> {
    if let Some(ip) = headers
        .get(&CF_CONNECTING_IP)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| IpValue::from_str(s).ok())
    {
        return Some(ip);
    }

    if let Some(ip) = headers
        .get(&X_FORWARDED_FOR)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| IpValue::from_str(s.trim()).ok())
    {
        return Some(ip);
    }

    headers.get(&X_REAL_IP).and_then(|v| v.to_str().ok()).and_then(|s| IpValue::from_str(s).ok())
}

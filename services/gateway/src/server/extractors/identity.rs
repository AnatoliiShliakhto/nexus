use crate::infra::ip_value::IpValue;
use axum::extract::{ConnectInfo, FromRequestParts, Request};
use http::request::Parts;
use http::{HeaderMap, HeaderName, Method, Uri, header};
use smol_str::SmolStr;
use std::net::SocketAddr;
use std::str::FromStr;

pub(crate) const CF_CONNECTING_IP: HeaderName = HeaderName::from_static("cf-connecting-ip");
pub(crate) const X_FORWARDED_FOR: HeaderName = HeaderName::from_static("x-forwarded-for");
pub(crate) const X_REAL_IP: HeaderName = HeaderName::from_static("x-real-ip");
pub(crate) const X_FORWARDED_PROTO: HeaderName = HeaderName::from_static("x-forwarded-proto");
pub(crate) const X_FORWARDED_HOST: HeaderName = HeaderName::from_static("x-forwarded-host");
pub(crate) const FORWARDED: HeaderName = HeaderName::from_static("forwarded");
pub(crate) const DPOP: HeaderName = HeaderName::from_static("dpop");
pub(crate) const X_DPOP_NONCE: HeaderName = HeaderName::from_static("x-dpop-nonce");

const AUTH_SCHEME_DPOP: &str = "dpop";
const DEFAULT_HTTP_SCHEME: &str = "http";

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
        let ip = extract_client_ip(headers).or_else(|| {
            extensions.get::<ConnectInfo<SocketAddr>>().map(|addr| IpValue::from(addr.0.ip()))
        });

        let session_id = extract_dpop_session_id(headers);
        let dpop_proof = header_boxed_str(headers, &DPOP);
        let dpop_nonce = header_smol_str(headers, &X_DPOP_NONCE);
        let user_agent = header_smol_str(headers, &header::USER_AGENT);
        let htu = build_dpop_htu(uri, headers);
        let htm = method.clone();

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

// --- DPoP HTU ---

#[inline]
fn build_dpop_htu(uri: &Uri, headers: &HeaderMap) -> SmolStr {
    let scheme =
        forwarded_proto(headers).or_else(|| uri.scheme_str()).unwrap_or(DEFAULT_HTTP_SCHEME);

    let authority = forwarded_host(headers)
        .or_else(|| host_header(headers))
        .or_else(|| uri.authority().map(|value| value.as_str()))
        .unwrap_or_default();

    let path = uri.path();

    let authority = normalize_authority(scheme, authority);

    let mut htu = String::with_capacity(scheme.len() + 3 + authority.len() + path.len());
    htu.push_str(scheme);
    htu.push_str("://");
    htu.push_str(authority);
    htu.push_str(path);

    SmolStr::from(htu)
}

#[inline]
fn forwarded_proto(headers: &HeaderMap) -> Option<&str> {
    forwarded_param(headers, "proto")
        .or_else(|| first_csv_header_value(headers, &X_FORWARDED_PROTO))
        .map(trim_ascii_whitespace)
        .filter(|value| !value.is_empty())
}

#[inline]
fn forwarded_host(headers: &HeaderMap) -> Option<&str> {
    forwarded_param(headers, "host")
        .or_else(|| first_csv_header_value(headers, &X_FORWARDED_HOST))
        .map(trim_ascii_whitespace)
        .filter(|value| !value.is_empty())
}

#[inline]
fn host_header(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(&header::HOST)
        .and_then(|value| value.to_str().ok())
        .map(trim_ascii_whitespace)
        .filter(|value| !value.is_empty())
}

fn forwarded_param<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    let raw = headers.get(&FORWARDED)?.to_str().ok()?;
    let first_forwarded = raw.split(',').next()?;

    for part in first_forwarded.split(';') {
        let (name, value) = part.trim().split_once('=')?;
        if name.trim().eq_ignore_ascii_case(key) {
            let value = trim_ascii_whitespace(value).trim_matches('"');
            if !value.is_empty() {
                return Some(value);
            }
        }
    }

    None
}

#[inline]
fn normalize_authority<'a>(scheme: &str, authority: &'a str) -> &'a str {
    if let Some(host) = authority.strip_suffix(":80") {
        if scheme.eq_ignore_ascii_case("http") {
            return host;
        }
    }

    if let Some(host) = authority.strip_suffix(":443") {
        if scheme.eq_ignore_ascii_case("https") {
            return host;
        }
    }

    authority
}

// --- Headers ---

#[inline]
fn header_smol_str(headers: &HeaderMap, name: &HeaderName) -> Option<SmolStr> {
    headers.get(name).and_then(|value| value.to_str().ok()).map(SmolStr::from)
}

#[inline]
fn header_boxed_str(headers: &HeaderMap, name: &HeaderName) -> Option<Box<str>> {
    headers.get(name).and_then(|value| value.to_str().ok()).map(Box::<str>::from)
}

#[inline]
fn extract_dpop_session_id(headers: &HeaderMap) -> Option<SmolStr> {
    let value = headers.get(&header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, credentials) = value.split_once(' ')?;

    if !scheme.eq_ignore_ascii_case(AUTH_SCHEME_DPOP) {
        return None;
    }

    let credentials = trim_ascii_whitespace(credentials);
    if credentials.is_empty() {
        return None;
    }

    Some(SmolStr::from(credentials))
}

// --- Client IP ---

#[inline]
fn extract_client_ip(headers: &HeaderMap) -> Option<IpValue> {
    if let Some(ip) = header_ip(headers, &CF_CONNECTING_IP) {
        return Some(ip);
    }

    if let Some(ip) = first_csv_ip(headers, &X_FORWARDED_FOR) {
        return Some(ip);
    }

    header_ip(headers, &X_REAL_IP)
}

#[inline]
fn header_ip(headers: &HeaderMap, name: &HeaderName) -> Option<IpValue> {
    let value = headers.get(name)?.to_str().ok()?;
    IpValue::from_str(trim_ascii_whitespace(value)).ok()
}

#[inline]
fn first_csv_ip(headers: &HeaderMap, name: &HeaderName) -> Option<IpValue> {
    let value = first_csv_header_value(headers, name)?;
    IpValue::from_str(trim_ascii_whitespace(value)).ok()
}

#[inline]
fn first_csv_header_value<'a>(headers: &'a HeaderMap, name: &HeaderName) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()?.split(',').next()
}

#[inline]
fn trim_ascii_whitespace(value: &str) -> &str {
    value.trim_matches(|ch: char| ch.is_ascii_whitespace())
}

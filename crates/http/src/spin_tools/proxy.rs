use nx_error::prelude::*;
use spin_sdk::http::Request;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use url::Url;

#[error]
pub enum ProxyRequestError {
    #[error(
        message = "The proxy target URI is invalid",
        status = ErrorStatus::InternalServerError,
        code = "HTTP_PROXY_TARGET_INVALID",
        source = url::ParseError,
    )]
    TargetInvalid,

    #[error(message = "Missing target URL", status = ErrorStatus::InternalServerError, code = "HTTP_PROXY_TARGET_MISSING")]
    TargetUrlMissing,

    #[error(message = "Failed to build request", status = ErrorStatus::InternalServerError, code = "HTTP_PROXY_BUILD_FAILED")]
    BuildFailed,
}

/// Headers that must be stripped according to RFC 2616 (Hop-by-Hop).
/// These manage the immediate connection and should never be forwarded to a backend.
const HOP_BY_HOP_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
    "host",
];

/// The strict allowlist of headers permitted to pass through when using strict proxying.
const ALLOWED_FORWARD_HEADERS: &[&str] =
    &["content-type", "content-length", "accept", "authorization"];

/// A builder for constructing a proxied HTTP request.
pub struct ProxyRequest<T> {
    request: Request<T>,
    target_url: Cow<'static, str>,
    strip_prefix: Option<Cow<'static, str>>,
    extra_headers: Vec<(Cow<'static, str>, Cow<'static, str>)>,
    exclude_headers: Vec<Cow<'static, str>>,
    strict_allowlist: bool,
}

impl<T> Debug for ProxyRequest<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyRequest")
            .field("request", &"(omitted for security reasons)")
            .field("target_url", &self.target_url)
            .field("strip_prefix", &self.strip_prefix)
            .field("extra_headers", &self.extra_headers)
            .field("exclude_headers", &self.exclude_headers)
            .field("strict_allowlist", &self.strict_allowlist)
            .finish()
    }
}

impl<T> ProxyRequest<T> {
    /// Creates a new `ProxyRequest` builder.
    pub fn new(request: Request<T>, url: impl Into<Cow<'static, str>>) -> Self {
        Self {
            request,
            target_url: url.into(),
            strip_prefix: None,
            extra_headers: Vec::new(),
            exclude_headers: Vec::new(),
            strict_allowlist: false,
        }
    }

    /// Sets a path prefix to be stripped from the original request URI.
    #[must_use = "The builder must be built to be used"]
    pub fn strip_prefix(mut self, prefix: impl Into<Cow<'static, str>>) -> Self {
        self.strip_prefix = Some(prefix.into());
        self
    }

    /// Adds or overwrites a header in the outgoing request.
    #[must_use = "The builder must be built to be used"]
    pub fn header(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.extra_headers.push((key.into(), value.into()));
        self
    }

    /// Excludes a header from the outgoing request.
    #[must_use = "The builder must be built to be used"]
    pub fn exclude_header(mut self, key: impl Into<Cow<'static, str>>) -> Self {
        self.exclude_headers.push(key.into());
        self
    }

    /// Enables a strict allowlist for headers, stripping everything except common safe headers.
    #[must_use = "The builder must be built to be used"]
    pub const fn strict_allowlist(mut self) -> Self {
        self.strict_allowlist = true;
        self
    }

    /// Consumes the builder and returns a configured [`Request<T>`].
    ///
    /// # Errors
    /// Returns [`ProxyError`] if the target URL is invalid or parsing fails.
    pub fn build(self) -> Result<Request<T>, ProxyRequestError> {
        if self.target_url.is_empty() {
            return Err(ProxyRequestError::target_url_missing());
        }

        let original_path = self.request.uri().path();
        let processed_path = self
            .strip_prefix
            .as_ref()
            .and_then(|prefix| original_path.strip_prefix(prefix.as_ref()))
            .unwrap_or(original_path);

        let mut full_url = self.target_url.trim_end_matches('/').to_owned();

        let suffix = processed_path.trim_start_matches('/');
        if !suffix.is_empty() {
            full_url.push('/');
            full_url.push_str(suffix);
        }

        if let Some(query) = self.request.uri().query() {
            full_url.push('?');
            full_url.push_str(query);
        }

        let target_url = Url::parse(&full_url)?;

        let mut builder =
            Request::builder().method(self.request.method().clone()).uri(target_url.as_str());

        let extra_keys: Vec<String> =
            self.extra_headers.iter().map(|(k, _)| k.to_lowercase()).collect();

        for (k, v) in self.request.headers() {
            let k_str = k.as_str();

            if self.should_strip_header(k_str, &extra_keys) {
                continue;
            }

            if let Ok(v_str) = v.to_str() {
                builder = builder.header(k_str, v_str);
            }
        }

        for (k, v) in self.extra_headers {
            builder = builder.header(k.as_ref(), v.as_ref());
        }

        builder.body(self.request.into_body()).map_err(|_| ProxyRequestError::build_failed())
    }

    fn should_strip_header(&self, key: &str, extra_keys: &[String]) -> bool {
        if HOP_BY_HOP_HEADERS.iter().any(|h| h.eq_ignore_ascii_case(key)) {
            return true;
        }

        if self.exclude_headers.iter().any(|e| e.eq_ignore_ascii_case(key)) {
            return true;
        }

        if extra_keys.iter().any(|ek| ek.eq_ignore_ascii_case(key)) {
            return true;
        }

        if self.strict_allowlist {
            return !ALLOWED_FORWARD_HEADERS.iter().any(|h| h.eq_ignore_ascii_case(key));
        }

        false
    }
}

// --- Extension Trait ---

/// Extension trait to provide proxying capabilities to [`Request`].
pub trait ProxyRequestExt<T> {
    fn proxy_to(self, uri: impl Into<Cow<'static, str>>) -> ProxyRequest<T>;
}

// impl ProxyRequestExt for Request {
//     /// Returns a [`ProxyRequest`] builder to proxy this request to the given URI.
//     fn proxy_to(self, uri: impl Into<Cow<'static, str>>) -> ProxyRequest {
//         ProxyRequest::new(self, uri)
//     }
// }

impl<T> ProxyRequestExt<T> for Request<T> {
    /// Returns a [`ProxyRequest`] builder to proxy this request to the given URI.
    fn proxy_to(self, uri: impl Into<Cow<'static, str>>) -> ProxyRequest<T> {
        ProxyRequest::new(self, uri)
    }
}

use crate::error::GatewayError;
use crate::server::extractors::identity;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Request, Uri, Version};
use http::header;
use opentelemetry::global;
use opentelemetry_http::HeaderInjector;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use url::Url;

static HOP_BY_HOP_HEADERS: [HeaderName; 12] = [
    header::CONNECTION,
    header::PROXY_AUTHENTICATE,
    header::PROXY_AUTHORIZATION,
    header::TE,
    header::TRANSFER_ENCODING,
    header::UPGRADE,
    header::AUTHORIZATION,
    identity::CF_CONNECTING_IP,
    identity::X_FORWARDED_FOR,
    identity::X_REAL_IP,
    identity::DPOP,
    identity::X_DPOP_NONCE,
];

pub(crate) struct ProxyRequest<'a, B> {
    request: Request<B>,
    target_base: &'a Url,
    strip_prefix: Option<&'a str>,
    extra_headers: Vec<(HeaderName, HeaderValue)>,
    strict_allowlist: bool,
}

impl<'a, B> ProxyRequest<'a, B> {
    pub(crate) const fn new(request: Request<B>, target: &'a Url) -> Self {
        Self {
            request,
            target_base: target,
            strip_prefix: None,
            extra_headers: Vec::new(),
            strict_allowlist: false,
        }
    }

    pub(crate) fn strip_prefix(mut self, prefix: &'a str) -> Self {
        self.strip_prefix = Some(prefix);
        self
    }

    pub(crate) fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.extra_headers.push((name, value));
        self
    }

    pub(crate) fn build(self) -> Result<Request<B>, GatewayError> {
        let Self { request, target_base, strip_prefix, extra_headers, strict_allowlist } = self;

        let (mut parts, body) = request.into_parts();

        let original_uri = &parts.uri;
        let path = original_uri.path();

        let processed_path = strip_prefix.and_then(|p| path.strip_prefix(p)).unwrap_or(path);

        let pq = match original_uri.query() {
            Some(q) => format!("{processed_path}?{q}"),
            None => processed_path.to_owned(),
        };

        let uri_builder =
            Uri::builder().scheme(target_base.scheme()).authority(target_base.authority());

        parts.uri =
            uri_builder.path_and_query(pq).build().map_err(|_| GatewayError::uri_parse_failed())?;

        parts.version = Version::HTTP_11;

        let mut filtered_headers =
            HeaderMap::with_capacity(parts.headers.len() + extra_headers.len());

        for (name, value) in parts.headers {
            if let Some(ref name) = name {
                if HOP_BY_HOP_HEADERS.contains(name) || name == header::HOST {
                    continue;
                }

                if strict_allowlist && !is_allowed_forward(name) {
                    continue;
                }
            }

            if let Some(name) = name {
                filtered_headers.append(name, value);
            }
        }

        for (name, value) in extra_headers {
            filtered_headers.insert(name, value);
        }

        parts.headers = filtered_headers;

        let context = tracing::Span::current().context();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&context, &mut HeaderInjector(&mut parts.headers));
        });

        Ok(Request::from_parts(parts, body))
    }
}

pub(crate) trait ProxyRequestExt<'a, B> {
    fn proxy_to(self, target: &'a Url) -> ProxyRequest<'a, B>;
}

impl<'a, B> ProxyRequestExt<'a, B> for Request<B> {
    fn proxy_to(self, target: &'a Url) -> ProxyRequest<'a, B> {
        ProxyRequest::new(self, target)
    }
}

#[inline]
fn is_allowed_forward(name: &HeaderName) -> bool {
    matches!(name.as_str(), "content-type" | "content-length" | "accept" | "traceparent")
}

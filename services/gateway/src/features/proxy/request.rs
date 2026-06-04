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
    strict_allowlist: bool,
}

impl<'a, B> ProxyRequest<'a, B> {
    pub(crate) const fn new(request: Request<B>, target: &'a Url) -> Self {
        Self { request, target_base: target, strip_prefix: None, strict_allowlist: false }
    }

    pub(crate) fn strip_prefix(mut self, prefix: &'a str) -> Self {
        self.strip_prefix = Some(prefix);
        self
    }

    #[inline]
    pub(crate) fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.request.headers_mut().insert(name, value);
        self
    }

    pub(crate) fn build(self) -> Result<Request<B>, GatewayError> {
        let Self { request, target_base, strip_prefix, strict_allowlist } = self;

        let (mut parts, body) = request.into_parts();

        let path_and_query = if let Some(prefix) = strip_prefix {
            let path = parts.uri.path();
            let processed_path = path.strip_prefix(prefix).unwrap_or(path);

            let pq_string = match parts.uri.query() {
                Some(q) => format!("{processed_path}?{q}"),
                None => processed_path.to_owned(),
            };
            pq_string.try_into().map_err(|_| GatewayError::uri_parse_failed())?
        } else {
            parts
                .uri
                .path_and_query()
                .cloned()
                .unwrap_or_else(|| http::uri::PathAndQuery::from_static("/"))
        };

        let mut uri_parts = parts.uri.into_parts();
        uri_parts.scheme =
            Some(target_base.scheme().parse().map_err(|_| GatewayError::uri_parse_failed())?);
        uri_parts.authority =
            Some(target_base.authority().parse().map_err(|_| GatewayError::uri_parse_failed())?);
        uri_parts.path_and_query = Some(path_and_query);

        parts.uri = Uri::from_parts(uri_parts).map_err(|_| GatewayError::uri_parse_failed())?;
        parts.version = Version::HTTP_11;

        if strict_allowlist {
            let mut filtered_headers = HeaderMap::with_capacity(parts.headers.len());
            for (name, value) in parts.headers {
                if let Some(ref name) = name {
                    if HOP_BY_HOP_HEADERS.contains(name) || name == header::HOST {
                        continue;
                    }
                    if !is_allowed_forward(name) {
                        continue;
                    }
                }
                if let Some(name) = name {
                    filtered_headers.append(name, value);
                }
            }
            parts.headers = filtered_headers;
        } else {
            for hop_header in &HOP_BY_HOP_HEADERS {
                parts.headers.remove(hop_header);
            }
            parts.headers.remove(header::HOST);
        }

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

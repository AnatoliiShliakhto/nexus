#![allow(unused_crate_dependencies)]

#[cfg(test)]
mod tests {
    use nx_http::http::StatusCode;
    use nx_http::spin_tools::proxy::{ProxyRequestError, ProxyRequestExt};
    use nx_http::trace::TraceContext;
    use nx_http::wasi::ErrorResponse;
    use spin_sdk::http::{Method, Request};

    #[test]
    fn test_trace_context_new_valid_format() {
        let ctx = TraceContext::new();
        let tp = ctx.traceparent();

        // Traceparent should follow W3C format: 00-traceid-parentid-flags
        assert_eq!(tp.len(), 55);
        assert!(tp.starts_with("00-"));
        assert_eq!(tp.chars().filter(|&c| c == '-').count(), 3);
    }

    #[test]
    fn test_trace_context_propagation() {
        let parent = TraceContext::new();
        let child = parent.child();

        // Child must share Trace ID but have a unique Parent (Span) ID
        assert_eq!(parent.trace_id(), child.trace_id());
        assert_ne!(parent.parent_id(), child.parent_id());
    }

    #[test]
    fn test_error_response_json_serialization() {
        let err_res =
            ErrorResponse::new(404, "USER_NOT_FOUND", "The requested user does not exist");

        let response = err_res.build();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers().get("content-type").unwrap(), "application/json");

        let body_str = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body_str.contains("\"status\":404"));
        assert!(body_str.contains("\"code\":\"USER_NOT_FOUND\""));
        assert!(body_str.contains("\"message\":\"The requested user does not exist\""));
    }

    #[test]
    fn test_rng_initialization() {
        let ctx1 = TraceContext::new();
        let ctx2 = TraceContext::new();
        // Subsequent calls should produce globally unique identifiers
        assert_ne!(ctx1.traceparent(), ctx2.traceparent());
    }

    /// Helper to quickly build a Request with specific headers for testing.
    fn make_request(uri: &str, headers: &[(&str, &str)]) -> Request<()> {
        let mut builder = Request::builder().method(Method::GET).uri(uri);

        for (k, v) in headers {
            builder = builder.header(*k, *v);
        }

        builder.body(()).expect("Failed to build request")
    }

    /// Helper to easily extract and assert headers from a Request.
    fn get_header_value<T>(req: &Request<T>, key: &str) -> Option<String> {
        req.headers().get(key).and_then(|v| v.to_str().ok()).map(ToOwned::to_owned)
    }

    #[test]
    fn test_proxy_builder_strips_hop_by_hop() -> Result<(), ProxyRequestError> {
        let req = make_request(
            "http://localhost:3000/original",
            &[
                ("Host", "localhost:3000"),
                ("Connection", "keep-alive"),
                ("X-Custom-Header", "allowed-value"),
            ],
        );

        let proxy_req = req.proxy_to("http://backend:8080").build()?;

        // Verify that hop-by-hop and empty headers are stripped
        assert_eq!(get_header_value(&proxy_req, "Host"), None);
        assert_eq!(get_header_value(&proxy_req, "Connection"), None);
        assert_eq!(get_header_value(&proxy_req, "Empty-Header"), None);

        // Verify that custom business headers remain intact
        assert_eq!(get_header_value(&proxy_req, "X-Custom-Header").unwrap(), "allowed-value");
        Ok(())
    }

    #[test]
    fn test_proxy_builder_strict_allowlist() -> Result<(), ProxyRequestError> {
        let req = make_request(
            "http://localhost:3000/original",
            &[
                ("Host", "localhost"),
                ("Content-Type", "application/json"),
                ("X-Proprietary-Data", "secret"),
            ],
        );

        let proxy_req = req
            .proxy_to("http://backend:8080")
            .header("X-Injected-Auth", "token-123")
            .strict_allowlist()
            .build()?;

        // In strict mode, unknown headers must be dropped even if they aren't hop-by-hop
        assert_eq!(get_header_value(&proxy_req, "X-Proprietary-Data"), None);
        assert_eq!(get_header_value(&proxy_req, "Host"), None);

        // Standard allowed headers should be preserved
        assert_eq!(get_header_value(&proxy_req, "Content-Type").unwrap(), "application/json");

        // Injected headers must be present
        assert_eq!(get_header_value(&proxy_req, "X-Injected-Auth").unwrap(), "token-123");
        Ok(())
    }

    #[test]
    fn test_proxy_builder_overwrites_existing_headers() -> Result<(), ProxyRequestError> {
        let req = make_request(
            "http://localhost:3000/original",
            &[("X-Request-Id", "old-id-123"), ("Authorization", "Bearer bad-token")],
        );

        let proxy_req =
            req.proxy_to("http://backend:8080").header("X-Request-Id", "new-id-456").build()?;

        // Check that the new header overwrote the old one without duplication
        assert_eq!(get_header_value(&proxy_req, "X-Request-Id").unwrap(), "new-id-456");

        // Ensure other headers remain unchanged
        assert_eq!(get_header_value(&proxy_req, "Authorization").unwrap(), "Bearer bad-token");
        Ok(())
    }

    #[test]
    fn test_proxy_builder_exclude_headers() -> Result<(), ProxyRequestError> {
        let req = make_request(
            "http://localhost:3000/original",
            &[("Cookie", "session=123"), ("Accept", "*/*")],
        );

        let proxy_req = req.proxy_to("http://backend:8080").exclude_header("Cookie").build()?;

        // Excluded headers must be removed from the final request
        assert_eq!(get_header_value(&proxy_req, "Cookie"), None);
        assert_eq!(get_header_value(&proxy_req, "Accept").unwrap(), "*/*");
        Ok(())
    }

    #[test]
    fn test_proxy_builder_strip_prefix_and_rebase() -> Result<(), ProxyRequestError> {
        let req = make_request("http://gateway.local/nexus/api/v1/users?active=true", &[]);

        // The target backend has its own base path "/internal"
        let proxy_req =
            req.proxy_to("http://internal-svc:8080/internal").strip_prefix("/nexus").build()?;

        // Expected URI: target_base + (original_path - prefix) + query
        // "http://internal-svc:8080/internal" + "/api/v1/users" + "?active=true"
        assert_eq!(proxy_req.uri(), "http://internal-svc:8080/internal/api/v1/users?active=true");
        Ok(())
    }

    #[test]
    fn test_proxy_builder_invalid_url_error() {
        let req = make_request("http://localhost/test", &[]);

        // Passing an invalid URL should return a ProxyRequestError::UrlParse
        let result = req.proxy_to("not-a-valid-url").build();
        assert!(result.is_err());
    }
}

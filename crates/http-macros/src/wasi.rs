use crate::helpers::{extract_error_type, generate_error_trace};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, ReturnType};

pub(crate) fn derive_service(func: &ItemFn) -> TokenStream {
    let sig = &func.sig;
    let name = &sig.ident;

    let error_type = if let ReturnType::Type(.., ty) = &func.sig.output {
        match extract_error_type(ty) {
            Some(ty) => ty,
            None => {
                return syn::Error::new_spanned(
                    func.sig.fn_token,
                    "the `#[wasi_service]` function must return `Result<impl IntoResponse, E>`",
                )
                .to_compile_error();
            },
        }
    } else {
        return syn::Error::new_spanned(
            func.sig.fn_token,
            "the `#[wasi_service]` function must return a `Result`",
        )
        .to_compile_error();
    };

    let error_trace = generate_error_trace();
    let mod_wasi = generate_wasi();

    quote! {
        fn _assert_error_impl_metadata() where #error_type: ::nx_http::error::ErrorMetadata {}
        #[allow(clippy::needless_pass_by_value)]
        #func

        #[derive(Debug)]
        struct Component;

        impl crate::bindings::exports::wasi::http::incoming_handler::Guest for Component {
            fn handle(
                in_request: crate::bindings::wasi::http::types::IncomingRequest,
                out_param: crate::bindings::wasi::http::types::ResponseOutparam,
            ) {
                use ::nx_http::error::{ErrorMetadata, ErrorMetadataExt};
                use ::nx_http::request::RequestHeadersExt;
                use ::nx_http::bytes::Bytes;
                use ::nx_http::http::HeaderValue;
                use ::nx_http::trace::TraceContext;

                ::nx_http::telemetry::init();

                let mut request = match _wasi::incoming_request_to_http(in_request) {
                    Ok(req) => req,
                    Err(e) => {
                        ::nx_http::tracing::error!(kind = "WASI_REQUEST_CONVERSION", "{e}");

                        let response =
                        ::nx_http::wasi::ErrorResponse::new(400, "WASI_REQUEST_CONVERSION", e).build();
                        _wasi::send_response(response, out_param).unwrap_or_else(|e| {
                            ::nx_http::tracing::error!("Critical failure sending response: {e}");
                        });
                        return;
                    },
                };
                let trace_ctx = request.header_value("traceparent").map_or_else(TraceContext::new, TraceContext::from);
                let span = ::nx_http::tracing::info_span!(
                    "http_request",
                    "service" = env!("CARGO_PKG_NAME"),
                    "http.method" = %request.method(),
                    "http.route" = %request.uri().path(),
                    "traceid" = ::nx_http::tracing::field::Empty,
                );
                let _guard = span.enter();
                let mut response = #name(request).unwrap_or_else(|e| {
                    ::nx_http::tracing::Span::current().record("traceid", trace_ctx.trace_id());
                    #error_trace
                    ::nx_http::http::Response::builder()
                    .status(e.status().as_u16())
                    .header("content-type", "application/json")
                    .body(Bytes::from(e.to_json_string()))
                    .expect("Failed to build error response")
                });
                let headers = response.headers_mut();
                if let Some(trace_id) = headers.get("x-trace-id") {
                    ::nx_http::tracing::Span::current().record("traceid", trace_id.to_str().unwrap_or(trace_ctx.trace_id()));
                } else {
                    let value = trace_ctx.trace_id().parse().unwrap_or_else(|_| HeaderValue::from_static(""));
                    headers.insert("x-trace-id", value);
                    ::nx_http::tracing::Span::current().record("traceid", trace_ctx.trace_id());
                }
                _wasi::send_response(response, out_param).unwrap_or_else(|e| {
                    ::nx_http::tracing::error!("Critical failure sending response: {e}");
                });
            }
        }

        crate::bindings::export!(Component with_types_in bindings);
        #mod_wasi
    }
}

fn generate_wasi() -> TokenStream {
    quote! {
        mod _wasi {
            use ::nx_http::bytes::Bytes;
            use ::nx_http::http::{
                HeaderMap, HeaderName, HeaderValue, Method as HttpMethod, Request, Response,
            };

            use crate::bindings::wasi::http::types::{
                Fields, IncomingRequest, Method, OutgoingBody, OutgoingResponse, ResponseOutparam, Scheme,
            };

            pub(super) fn incoming_request_to_http(
                in_request: IncomingRequest,
            ) -> Result<Request<Bytes>, &'static str> {
                let mut headers = HeaderMap::new();
                for (name, value) in in_request.headers().entries() {
                    let h_name = HeaderName::from_bytes(name.as_bytes())
                    .map_err(|_| "Invalid request header name")?;
                    let h_val =
                    HeaderValue::from_bytes(&value).map_err(|_| "Invalid request header value")?;
                    headers.insert(h_name, h_val);
                }

                let body_handle = in_request.consume().map_err(|_| "Failed to consume request body")?;

                let stream = body_handle.stream().map_err(|_| "Failed to get request body stream")?;

                let mut body = Vec::new();
                while let Ok(chunk) = stream.read(1024 * 64) {
                    if chunk.is_empty() {
                        break;
                    }
                    body.extend_from_slice(&chunk);
                }

                let scheme = match in_request.scheme() {
                    Some(Scheme::Http) => "http",
                    Some(Scheme::Https) => "https",
                    Some(Scheme::Other(ref s)) => s.as_str(),
                    None => "http",
                }.to_owned();
                let mut builder = ::nx_http::http::Uri::builder();
                if let Some(s) = in_request.scheme() { builder = builder.scheme(scheme.as_str()); }
                if let Some(a) = in_request.authority() { builder = builder.authority(a); }
                if let Some(pq) = in_request.path_with_query() { builder = builder.path_and_query(pq); }
                let uri = builder.build().map_err(|_| "Invalid URI")?;

                let mut builder = Request::builder()
                .method(convert_method(in_request.method()))
                .uri(uri);

                *builder.headers_mut().unwrap() = headers;

                builder.body(Bytes::copy_from_slice(&body)).map_err(|_| "Failed to build request")
            }

            pub(super) fn send_response(
                out_response: Response<Bytes>,
                out_param: ResponseOutparam,
            ) -> Result<(), &'static str> {
                let headers = Fields::new();
                for (name, value) in out_response.headers().iter() {
                    headers
                    .append(name.as_str(), value.as_bytes())
                    .map_err(|_| "Failed to append response header")?;
                }

                let response = OutgoingResponse::new(headers);
                response
                .set_status_code(out_response.status().as_u16())
                .map_err(|_| "Failed to set status code")?;

                let body = response.body().map_err(|_| "Failed to get response body")?;
                let stream = body.write().map_err(|_| "Failed to get response body stream")?;

                stream
                .blocking_write_and_flush(out_response.body().as_ref())
                .map_err(|_| "Failed to write response body")?;

                drop(stream);
                OutgoingBody::finish(body, None).map_err(|_| "Failed to finish response body")?;

                ResponseOutparam::set(out_param, Ok(response));
                Ok(())
            }

            fn convert_method(method: Method) -> HttpMethod {
                match method {
                    Method::Get => HttpMethod::GET,
                    Method::Post => HttpMethod::POST,
                    Method::Put => HttpMethod::PUT,
                    Method::Delete => HttpMethod::DELETE,
                    Method::Head => HttpMethod::HEAD,
                    Method::Options => HttpMethod::OPTIONS,
                    Method::Connect => HttpMethod::CONNECT,
                    Method::Patch => HttpMethod::PATCH,
                    Method::Trace => HttpMethod::TRACE,
                    Method::Other(s) => HttpMethod::from_bytes(s.as_bytes()).unwrap_or(HttpMethod::GET),
                }
            }
        }
    }
}

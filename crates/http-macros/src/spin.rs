use crate::helpers::{extract_error_type, generate_error_trace};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, ItemFn, Pat, ReturnType, Token};

#[allow(clippy::too_many_lines)]
pub(crate) fn derive_handler(func: &ItemFn) -> TokenStream {
    let sig = &func.sig;
    let name = &sig.ident;

    if func.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            func.sig.fn_token,
            "the `#[spin_service]` function must be `async`",
        )
        .to_compile_error();
    }
    let error_type = if let ReturnType::Type(.., ty) = &func.sig.output {
        match extract_error_type(ty) {
            Some(ty) => ty,
            None => {
                return syn::Error::new_spanned(
                    func.sig.fn_token,
                    "the `#[spin_service]` function must return `Result<Response, E>` or `Result<impl IntoResponse, E>`",
                )
                .to_compile_error();
            },
        }
    } else {
        return syn::Error::new_spanned(
            func.sig.fn_token,
            "the `#[spin_service]` function must return a `Result`",
        )
        .to_compile_error();
    };

    let error_trace = generate_error_trace();

    let module = quote! {
        mod __spin_wasip3_http_new {
            use ::nx_http::spin_sdk::http::{IntoResponse, FromRequest, Request, Response};
            use ::nx_http::spin_sdk::wasip3::exports::http::handler::Guest;
            use ::nx_http::spin_sdk::wasip3::http::types::{Request as IncomingRequest, Response as OutgoingResponse, ErrorCode};
            use ::nx_http::spin_sdk::wasip3::http::service::export;
            use ::nx_http::error::{ErrorMetadata, ErrorMetadataExt};
            use ::nx_http::request::RequestHeadersExt;
            use ::nx_http::tracing::Instrument;
            use ::nx_http::trace::TraceContext;

            struct Spin;
            export!(Spin);

            fn _assert_error_impl_metadata() where super::#error_type: ::nx_http::error::ErrorMetadata {}
            impl Guest for self::Spin {
                async fn handle(request: IncomingRequest) -> Result<OutgoingResponse, ErrorCode> {
                    ::nx_http::telemetry::init();
                    let request = <Request as FromRequest>::from_request(request)?;
                    let trace_ctx = request.header_value("traceparent").map_or_else(TraceContext::new, TraceContext::from);
                    let span = ::nx_http::tracing::info_span!(
                        "http_request",
                        "service" = env!("CARGO_PKG_NAME"),
                        "http.method" = %request.method(),
                        "http.uri" = %request.uri(),
                        "trace_id" = ::nx_http::tracing::field::Empty,
                    );
                    async move {
                        match super::#name(request).await {
                            Ok(response) => response.into_response(),
                            Err(e) => {
                                ::nx_http::tracing::Span::current().record("trace_id", trace_ctx.trace_id());
                                #error_trace
                                Response::builder()
                                .status(e.status().as_u16())
                                .header("content-type", "application/json")
                                .body(e.to_json_string())
                                .unwrap_or_else(|_| {
                                    Response::builder()
                                    .status(500)
                                    .body(String::new())
                                    .expect("Failed to create response")
                                })
                                .into_response()
                            }
                        }.map(|r| {
                            let headers = r.get_headers();
                            if headers.has("x-trace-id") {
                                let trace_id = headers
                                .get("x-trace-id")
                                .first()
                                .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
                                .unwrap_or_else(|| trace_ctx.trace_id().to_owned());
                                ::nx_http::tracing::Span::current().record("trace_id", trace_id);
                            } else {
                                let _ = headers.set(
                                    &"x-trace-id".to_string(),
                                    &[trace_ctx.trace_id().as_bytes().to_vec()]
                                );
                                ::nx_http::tracing::Span::current().record("trace_id", trace_ctx.trace_id());
                            }
                            r
                        })
                    }.instrument(span).await
                }
            }
        }
    };

    let expanded = quote! {
        #func
        #module
    };

    expanded
}

// --- Router ---

#[derive(Debug)]
struct RouteArm {
    method: Ident,
    path_pattern: Pat,
    _fat_arrow: Token![=>],
    handler: Expr,
}

#[derive(Debug)]
pub(crate) struct RouterInput {
    request_expr: Expr,
    _comma: Token![,],
    routes: Vec<RouteArm>,
    _underscore: Token![_],
    _fallback_arrow: Token![=>],
    not_found_expr: Expr,
}

impl Parse for RouteArm {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            method: input.parse()?,
            path_pattern: Pat::parse_single(input)?,
            _fat_arrow: input.parse()?,
            handler: input.parse()?,
        })
    }
}

impl Parse for RouterInput {
    #[allow(clippy::used_underscore_binding)]
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let request_expr = input.parse()?;
        let _comma = input.parse()?;

        let mut routes = Vec::new();

        while !input.peek(Token![_]) {
            routes.push(input.parse()?);
            if input.peek(Token![,]) {
                let _skip_comma: Token![,] = input.parse()?;
            }
        }

        let _underscore = input.parse()?;
        let _fallback_arrow = input.parse()?;
        let not_found_expr = input.parse()?;

        if input.peek(Token![,]) {
            let _skip_final_comma: Token![,] = input.parse()?;
        }

        Ok(Self { request_expr, _comma, routes, _underscore, _fallback_arrow, not_found_expr })
    }
}

pub(crate) fn router_derive(input: &RouterInput) -> TokenStream {
    let req = &input.request_expr;
    let not_found = &input.not_found_expr;

    let arms = input.routes.iter().map(|arm| {
        let method_span = arm.method.span();
        let method_str = arm.method.to_string();

        let method_upper = method_str.to_uppercase();
        let method_ident = Ident::new(&method_upper, method_span);

        let pattern = &arm.path_pattern;
        let handler = &arm.handler;

        quote! {
            (&nx_http::spin_sdk::http::Method::#method_ident, #pattern) => #handler.map(::nx_http::spin_sdk::http::IntoResponse::into_response),
        }
    });

    let expanded = quote! {
        {
            let __path = #req.uri().path().trim_matches('/');
            let __segments: Vec<&str> = if __path.is_empty() {
                Vec::new()
            } else {
                __path.split('/').collect()
            };

            match (#req.method(), __segments.as_slice()) {
                #( #arms )*
                _ => #not_found,
            }
        }
    };

    expanded
}

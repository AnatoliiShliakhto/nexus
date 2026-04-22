use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

pub(crate) fn generate_error_trace() -> TokenStream {
    let gen_log = |level: &str| {
        let ident = syn::Ident::new(level, proc_macro2::Span::call_site());
        quote! {
            if detailed {
                ::nx_http::tracing::#ident!(
                    code = %e.code(),
                    details = %e.details().as_deref().unwrap_or(""),
                    help = %e.help().as_deref().unwrap_or(""),
                    caused_by = %e.to_detailed_json_string(),
                    "{}", e.message()
                );
            } else {
                ::nx_http::tracing::#ident!(
                    code = %e.code(),
                    details = %e.details().as_deref().unwrap_or(""),
                    "{}", e.message()
                );
            }
        }
    };

    let trace_log = gen_log("trace");
    let warn_log = gen_log("warn");
    let error_log = gen_log("error");

    quote! {
        let detailed = ::std::env::var("DETAILED_ERROR")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);
        let status = e.status().as_u16();

        match status {
            ..400 => { #trace_log },
            400..500 => { #warn_log },
            _ => { #error_log },
        }
    }
}

pub(crate) fn extract_error_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(tp) = ty {
        let last_segment = tp.path.segments.last()?;
        if last_segment.ident == "Result"
            && let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
            && args.args.len() == 2
            && let syn::GenericArgument::Type(error_ty) = &args.args[1]
        {
            return Some(error_ty);
        }
    }
    None
}

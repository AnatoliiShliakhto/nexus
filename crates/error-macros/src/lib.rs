mod generation;
mod parsing;
mod transform;

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{ItemEnum, parse_macro_input, parse_quote};

/// A high-performance, Wasm-optimized error framework for the Nexus ecosystem.
///
/// This macro transforms a standard Rust enum into a "Fortress-First" error type. It minimizes
/// boilerplate while providing rich metadata, telemetry-friendly serialization, and
/// zero-cost transparent delegation.
///
/// # Key Features
///
/// * **Metadata Chaining**: Replaces expensive backtraces with lightweight metadata (Status, Code, Message, Help).
/// * **Transparent Delegation**: Use `#[transparent(SourceType)]` to wrap upstream errors while
///   inheriting their metadata (status, code, etc.) without runtime overhead.
/// * **Zero-Copy Architecture**: Uses `Cow<'static, str>` for messages and details to optimize memory
///   usage in Wasm/WASI environments.
/// * **Compile-time Optimization**: Metadata resolution (deciding between attribute values or
///   source values) is performed during macro expansion.
///
/// # Generated API
///
/// For an enum named `ServiceError`, the macro generates:
///
/// * **`ServiceErrorData`**: A hidden internal struct for state management.
/// * **`ServiceErrorExt`**: An extension trait for `Result<T, ServiceError>` providing:
///   * `.with_message(...)`: Contextual override of the error message.
///   * `.with_details(...)`: Additional debugging context.
///   * `.with_help(...)`: Troubleshooting advice for the end-user.
/// * **Constructors**: Automatic `snake_case` constructors (e.g., `ServiceError::not_found()`).
///
/// # Attributes Reference
///
/// ### Variant Attributes (`#[error(...)]`)
/// * `status = u16`: HTTP-compatible status code (e.g., `404`, `500`).
/// * `code = "..."`: Machine-readable unique identifier (defaults to `SHOUTY_SNAKE_CASE`).
/// * `message = "..."`: Human-readable default description.
/// * `error_source = T`: Wraps an upstream error of type `T`.
/// * `help = "..."`: Troubleshooting instructions.
///
/// ### Transparent Attribute (`#[transparent(T)]`)
/// * Delegates all `ErrorMetadata` calls to the underlying error `T`.
/// * You can still override specific fields: `#[transparent(T), error(status = 403)]`.
///
/// # Example: Transparent Delegation
///
/// ```rust
/// #[nx_error::error]
/// pub enum ServiceError {
///     // Automatically inherits status (404), code, and message from DatabaseError
///     #[transparent(DatabaseError)]
///     Db,
///
///     // Standard wrapping: manually define metadata for external errors
///     #[error(status = 500, message = "System IO failure", code = "IO_ERROR", source = std::io::Error)]
///     Io,
/// }
/// ```
///
/// # Example: Result Enrichment
///
/// ```rust
/// use nx_error::ServiceErrorExt;
///
/// fn perform_task() -> Result<(), ServiceError> {
///     do_io()
///         .with_message("Message to display to the user")
///         .with_details("Detailed information for debugging")
///         .with_help("Helpful tips for troubleshooting")
/// }
/// ```
#[proc_macro_attribute]
pub fn error(args: TokenStream, input: TokenStream) -> TokenStream {
    let base_path = if args.is_empty() {
        get_base_path()
    } else {
        let path_str = args.to_string();
        path_str.parse().unwrap_or_else(|_| quote!(::nx_error))
    };

    expand_derive(input, &base_path)
}

fn expand_derive(input: TokenStream, base_path: &proc_macro2::TokenStream) -> TokenStream {
    let mut item_enum = parse_macro_input!(input as ItemEnum);

    let variants_info = match parsing::parse_and_clean_variants(&mut item_enum) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    item_enum.attrs.push(parse_quote!(#[derive(Debug)]));

    transform::transform_enum_variants(&mut item_enum, &variants_info, base_path);

    let impl_blocks =
        generation::generate_implementations(&item_enum.ident, &variants_info, base_path);

    let expanded = quote! {
        #[allow(unreachable_pub)]
        #item_enum
        #impl_blocks
    };

    TokenStream::from(expanded)
}

fn get_base_path() -> proc_macro2::TokenStream {
    if let Ok(FoundCrate::Name(name)) = crate_name("nx-http") {
        let ident = format_ident!("{name}");
        return quote!(::#ident::error);
    }

    if let Ok(FoundCrate::Name(name)) = crate_name("nx-error") {
        let ident = format_ident!("{name}");
        return quote!(::#ident);
    }

    quote!(::nx_error)
}

use super::parsing::ErrorVariantInfo;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

pub(super) fn generate_implementations(
    enum_name: &Ident,
    infos: &[ErrorVariantInfo],
    base_path: &TokenStream,
) -> TokenStream {
    let data_struct_def = generate_data_struct(enum_name, base_path);
    let adapter_def = generate_adapter(enum_name, infos, base_path);
    let error_metadata_impl = generate_error_metadata(enum_name, infos, base_path);
    let display_impl = generate_display(enum_name, base_path);
    let std_error_impl = generate_std_error(enum_name, infos);
    let constructors = generate_constructors(enum_name, infos, base_path);
    let mutators = generate_mutators(enum_name, infos);
    let trait_def_and_impl = generate_trait_ext(enum_name);
    let from_impls = generate_from_impls(enum_name, infos, base_path);

    quote! {
        #data_struct_def
        #adapter_def
        #constructors
        #mutators
        #std_error_impl
        #error_metadata_impl
        #display_impl
        #trait_def_and_impl
        #from_impls
    }
}

fn generate_data_struct(enum_name: &Ident, base_path: &TokenStream) -> TokenStream {
    let data_name = format_ident!("{enum_name}Data");
    quote! {
        #[doc(hidden)]
        #[allow(unreachable_pub)]
        #[derive(Debug)]
        pub struct #data_name {
            pub(crate) status: #base_path::ErrorStatus,
            pub(crate) code: &'static str,
            pub(crate) message: ::std::borrow::Cow<'static, str>,
            pub(crate) details: ::std::option::Option<::std::borrow::Cow<'static, str>>,
            pub(crate) help: ::std::option::Option<::std::borrow::Cow<'static, str>>,
        }
    }
}

fn generate_constructors(
    enum_name: &Ident,
    infos: &[ErrorVariantInfo],
    base_path: &TokenStream,
) -> TokenStream {
    let data_name = format_ident!("{enum_name}Data");

    let methods = infos.iter().map(|info| {
        let variant_ident = &info.ident;
        let fn_name = format_ident!("{}", variant_ident.to_string().to_snake_case());
        let cfg = &info.cfg_attrs;

        if info.source.is_some() {
            quote! {}
        } else {
            let status = &info.status;
            let code = &info.code;
            let msg = &info.message;
            let help = info.help.as_ref().map_or_else(
                || quote! { ::std::option::Option::None },
                |h| quote! { Some(::std::borrow::Cow::Borrowed(#h)) },
            );

            quote! {
                #(#cfg)*
                pub fn #fn_name() -> Self {
                    Self::#variant_ident {
                        data: Box::new(#data_name {
                            status: #base_path::ErrorStatus::from(#status),
                            code: #code,
                            message: #msg.into(),
                            details: ::std::option::Option::None,
                            help: #help,
                        })
                    }
                }
            }
        }
    });

    quote! {
        impl #enum_name {
            #(#methods)*
        }
    }
}

fn generate_mutators(enum_name: &Ident, infos: &[ErrorVariantInfo]) -> TokenStream {
    let message_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.message = message.into(), }
    });
    let details_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.details = Some(details.into()), }
    });
    let help_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.help = Some(help.into()), }
    });

    quote! {
        impl #enum_name {
            #[inline]
            pub fn with_message(mut self, message: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                match &mut self { #(#message_arms)* }
                self
            }
            #[inline]
            pub fn with_message_fn<F>(mut self, f: F) -> Self where F: FnOnce() -> String {
                self.with_message(f())
            }
            #[inline]
            pub fn with_details(mut self, details: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                match &mut self { #(#details_arms)* }
                self
            }
            #[inline]
            pub fn with_details_fn<F>(mut self, f: F) -> Self where F: FnOnce() -> String {
                self.with_details(f())
            }
            #[inline]
            pub fn with_help(mut self, help: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                match &mut self { #(#help_arms)* }
                self
            }
            #[inline]
            pub fn with_help_fn<F>(mut self, f: F) -> Self where F: FnOnce() -> String {
                self.with_help(f())
            }
        }
    }
}

fn generate_std_error(enum_name: &Ident, infos: &[ErrorVariantInfo]) -> TokenStream {
    let match_arms = infos.iter().map(|info| {
        let variant = &info.ident;
        let cfg = &info.cfg_attrs;
        if info.source.is_some() {
            quote! { #(#cfg)* #enum_name::#variant { source, .. } => Some(source.as_ref()), }
        } else {
            quote! { #(#cfg)* #enum_name::#variant { .. } => ::std::option::Option::None, }
        }
    });

    quote! {
        impl ::std::error::Error for #enum_name {
            fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error + 'static)> {
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}

fn generate_error_metadata(
    enum_name: &Ident,
    infos: &[ErrorVariantInfo],
    base_path: &TokenStream,
) -> TokenStream {
    let message_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.message.clone(), }
    });
    let code_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.code, }
    });
    let status_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.status, }
    });
    let details_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.details.clone(), }
    });
    let help_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.help.clone(), }
    });
    let source_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        if i.source.is_some() {
            quote! { #(#cfg)* Self::#v { source, .. } => Some(source.as_ref() as &(dyn #base_path::ErrorMetadata + 'static)), }
        } else {
            quote! { #(#cfg)* Self::#v { .. } => ::std::option::Option::None, }
        }
    });

    quote! {
        impl #base_path::ErrorMetadata for #enum_name {
            fn status(&self) -> #base_path::ErrorStatus {
                match self { #(#status_arms)* }
            }
            fn code(&self) -> &'static str {
                match self { #(#code_arms)* }
            }
            fn message(&self) -> ::std::borrow::Cow<'static, str> {
                match self { #(#message_arms)* }
            }
            fn details(&self) -> ::std::option::Option<::std::borrow::Cow<'static, str>> {
                match self { #(#details_arms)* }
            }
            fn error_source(&self) -> ::std::option::Option<&(dyn #base_path::ErrorMetadata + 'static)> {
                match self { #(#source_arms)* }
            }
            fn help(&self) -> ::std::option::Option<::std::borrow::Cow<'static, str>> {
                match self { #(#help_arms)* }
            }
            fn target(&self) -> &'static str {
                env!("CARGO_PKG_NAME")
            }
        }
    }
}

fn generate_display(enum_name: &Ident, base_path: &TokenStream) -> TokenStream {
    quote! {
        impl ::std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", <Self as #base_path::ErrorMetadata>::message(self))
            }
        }
    }
}

fn generate_trait_ext(enum_name: &Ident) -> TokenStream {
    let trait_name = format_ident!("{enum_name}Ext");
    quote! {
        pub trait #trait_name<T> {
            fn with_message(self, message: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name>;
            fn with_message_fn<F>(self, f: F) -> ::std::result::Result<T, #enum_name> where F: FnOnce() -> String;
            fn with_details(self, details: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name>;
            fn with_details_fn<F>(self, f: F) -> ::std::result::Result<T, #enum_name> where F: FnOnce() -> String;
            fn with_help(self, help: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name>;
            fn with_help_fn<F>(self, f: F) -> ::std::result::Result<T, #enum_name> where F: FnOnce() -> String;
        }

        impl<T, E> #trait_name<T> for ::std::result::Result<T, E>
            where
            E: ::std::convert::Into<#enum_name>,
        {
            #[inline]
            fn with_message(self, message: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name> {
                self.map_err(|e| ::std::convert::Into::<#enum_name>::into(e).with_message(message))
            }
            #[inline]
            fn with_message_fn<F>(self, f: F) -> ::std::result::Result<T, #enum_name> where F: FnOnce() -> String {
                self.map_err(|e| ::std::convert::Into::<#enum_name>::into(e).with_message_fn(f))
            }
            #[inline]
            fn with_details(self, details: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name> {
                self.map_err(|e| ::std::convert::Into::<#enum_name>::into(e).with_details(details))
            }
            #[inline]
            fn with_details_fn<F>(self, f: F) -> ::std::result::Result<T, #enum_name> where F: FnOnce() -> String {
                self.map_err(|e| ::std::convert::Into::<#enum_name>::into(e).with_details_fn(f))
            }
            #[inline]
            fn with_help(self, help: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name> {
                self.map_err(|e| ::std::convert::Into::<#enum_name>::into(e).with_help(help))
            }
            #[inline]
            fn with_help_fn<F>(self, f: F) -> ::std::result::Result<T, #enum_name> where F: FnOnce() -> String {
                self.map_err(|e| ::std::convert::Into::<#enum_name>::into(e).with_help_fn(f))
            }
        }
    }
}

fn generate_adapter(
    enum_name: &syn::Ident,
    infos: &[ErrorVariantInfo],
    base_path: &TokenStream,
) -> TokenStream {
    let adapter_name = format_ident!("{enum_name}Adapter");

    let markers = infos.iter().filter(|info| info.source.is_some()).map(|info| {
        let marker = format_ident!("__{enum_name}{}Marker", info.ident);
        let cfg = &info.cfg_attrs;
        quote! {
            #(#cfg)*
            #[doc(hidden)]
            #[allow(unreachable_pub)]
            #[derive(Debug)]
            pub(crate) struct #marker;
        }
    });

    let impls = infos.iter()
        .filter(|info| info.source.is_some())
        .map(|info| {
            let variant_ident = &info.ident;
            let marker = format_ident!("__{enum_name}{variant_ident}Marker");
            let cfg = &info.cfg_attrs;

            if info.transparent {
                let status_val = &info.status;
                let status_body = if quote!(#status_val).to_string() == "0" {
                    quote! { #base_path::ErrorMetadata::status(&self.0) }
                } else {
                    quote! { #base_path::ErrorStatus::from(#status_val) }
                };

                let code_val = &info.code;
                let code_body = if code_val.is_empty() {
                    quote! { #base_path::ErrorMetadata::code(&self.0) }
                } else {
                    quote! { #code_val }
                };

                let msg_val = &info.message;
                let msg_body = if msg_val.is_empty() {
                    quote! { #base_path::ErrorMetadata::message(&self.0) }
                } else {
                    quote! { #msg_val.into() }
                };

                let help_body = info.help.as_ref().map_or_else(
                    || quote! { #base_path::ErrorMetadata::help(&self.0) },
                    |h| quote! { ::std::option::Option::Some(::std::borrow::Cow::Borrowed(#h))
                    });

                quote! {
                    #(#cfg)*
                    impl<E: #base_path::ErrorMetadata + 'static> #base_path::ErrorMetadata for #adapter_name<E, #marker> {
                        #[inline] fn status(&self) -> #base_path::ErrorStatus { #status_body }
                        #[inline] fn code(&self) -> &'static str { #code_body }
                        #[inline] fn message(&self) -> ::std::borrow::Cow<'static, str> { #msg_body }
                        #[inline] fn help(&self) -> ::std::option::Option<::std::borrow::Cow<'static, str>> { #help_body }
                        fn target(&self) -> &'static str { #base_path::ErrorMetadata::target(&self.0) }
                        fn details(&self) -> ::std::option::Option<::std::borrow::Cow<'static, str>> { #base_path::ErrorMetadata::details(&self.0) }
                        fn error_source(&self) -> ::std::option::Option<&(dyn #base_path::ErrorMetadata + 'static)> {
                            #base_path::ErrorMetadata::error_source(&self.0)
                        }
                    }
                }
            } else {
                let status = &info.status;
                let code = &info.code;
                let msg = &info.message;
                let help = info.help.as_ref().map_or_else(
                    || quote! { ::std::option::Option::None },
                    |h| quote! { ::std::option::Option::Some(::std::borrow::Cow::Borrowed(#h)) }
                );

                quote! {
                    #(#cfg)*
                    impl<E: ::std::error::Error + 'static> #base_path::ErrorMetadata for #adapter_name<E, #marker> {
                        #[inline] fn status(&self) -> #base_path::ErrorStatus { #base_path::ErrorStatus::from(#status) }
                        #[inline] fn code(&self) -> &'static str { #code }
                        #[inline] fn message(&self) -> ::std::borrow::Cow<'static, str> { #msg.into() }
                        #[inline] fn help(&self) -> ::std::option::Option<::std::borrow::Cow<'static, str>> { #help }
                        fn details(&self) -> ::std::option::Option<::std::borrow::Cow<'static, str>> {
                            ::std::option::Option::Some(::std::borrow::Cow::Owned(format!("{}", self.0)))
                        }
                        fn target(&self) -> &'static str { env!("CARGO_PKG_NAME") }
                        fn error_source(&self) -> ::std::option::Option<&(dyn #base_path::ErrorMetadata + 'static)> { ::std::option::Option::None }
                    }
                }
            }
        });

    quote! {
        #[doc(hidden)]
        #[allow(unreachable_pub)]
        pub(crate) struct #adapter_name<E, M>(pub E, ::std::marker::PhantomData<M>);
        impl<E, M> #adapter_name<E, M> {
            #[inline] pub fn new(e: E) -> Self { Self(e, ::std::marker::PhantomData) }
        }
        #(#markers)*
        #(#impls)*
        impl<E: ::std::fmt::Display, M> ::std::fmt::Display for #adapter_name<E, M> {
            #[inline] fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result { self.0.fmt(f) }
        }
        impl<E: ::std::fmt::Debug, M> ::std::fmt::Debug for #adapter_name<E, M> {
            #[inline] fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result { self.0.fmt(f) }
        }
        impl<E: ::std::error::Error + 'static, M> ::std::error::Error for #adapter_name<E, M> {
            #[inline] fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error + 'static)> { self.0.source() }
        }
    }
}

fn generate_from_impls(
    enum_name: &syn::Ident,
    infos: &[ErrorVariantInfo],
    base_path: &TokenStream,
) -> TokenStream {
    let mut seen_signatures = fxhash::FxHashSet::default();
    let adapter_name = format_ident!("{enum_name}Adapter");
    let mut all_impls = Vec::new();

    for info in infos.iter().filter(|i| i.source.is_some()) {
        let source_type = info.source.as_ref().unwrap();
        let cfg = &info.cfg_attrs;
        let marker = format_ident!("__{enum_name}{}Marker", info.ident);

        let type_str = quote!(#source_type).to_string();
        let cfg_str = quote!(#(#cfg)*).to_string();
        let signature = format!("{cfg_str}|{type_str}");

        if seen_signatures.insert(signature) {
            let from_body = generate_variant_init_logic(enum_name, info, base_path);
            all_impls.push(quote! {
                #(#cfg)*
                impl ::std::convert::From<#source_type> for #enum_name {
                    #[inline]
                    fn from(source: #source_type) -> Self {
                        let adapter = #adapter_name::<_, #marker>::new(source);
                        #from_body
                    }
                }
            });
        }

        for extra_type in &info.extra_from {
            let extra_type_str = quote!(#extra_type).to_string();
            let signature = format!("{cfg_str}|{extra_type_str}");

            if seen_signatures.insert(signature) {
                all_impls.push(quote! {
                    #(#cfg)*
                    impl ::std::convert::From<#extra_type> for #enum_name {
                        #[inline]
                        fn from(source: #extra_type) -> Self {
                            Self::from(#source_type::from(source))
                        }
                    }
                });
            }
        }
    }

    quote! { #(#all_impls)* }
}

fn generate_variant_init_logic(
    enum_name: &Ident,
    info: &ErrorVariantInfo,
    base_path: &TokenStream,
) -> TokenStream {
    let variant_ident = &info.ident;
    let data_name = format_ident!("{enum_name}Data");

    quote! {
        let status = #base_path::ErrorMetadata::status(&adapter);
        let code = #base_path::ErrorMetadata::code(&adapter);
        let message = #base_path::ErrorMetadata::message(&adapter);
        let help = #base_path::ErrorMetadata::help(&adapter);
        let details = #base_path::ErrorMetadata::details(&adapter);

        Self::#variant_ident {
            data: Box::new(#data_name {
                status,
                code,
                message,
                details,
                help,
            }),
            source: Box::new(adapter) as Box<dyn #base_path::ErrorMetadata + ::core::marker::Send + ::core::marker::Sync>,
        }
    }
}

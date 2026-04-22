use super::parsing::ErrorVariantInfo;
use syn::{Fields, FieldsNamed, ItemEnum, parse_quote};

pub(super) fn transform_enum_variants(
    item: &mut ItemEnum,
    infos: &[ErrorVariantInfo],
    base_path: &proc_macro2::TokenStream,
) {
    let data_struct_name = quote::format_ident!("{}Data", item.ident);

    for (variant, info) in item.variants.iter_mut().zip(infos.iter()) {
        let source_field = info.source.as_ref().map_or_else(
            || quote::quote! {},
            |_src_type| quote::quote! { source: Box<dyn #base_path::ErrorMetadata + ::core::marker::Send + ::core::marker::Sync>, },
        );

        let fields: FieldsNamed = parse_quote! {
            {
                #source_field
                data: Box<#data_struct_name>,
            }
        };

        variant.fields = Fields::Named(fields);
    }
}

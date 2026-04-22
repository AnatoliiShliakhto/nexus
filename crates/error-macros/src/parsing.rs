use heck::{ToShoutySnakeCase, ToSnakeCase};
use syn::parse::Parse;
use syn::{
    Attribute, Error, Expr, Fields, Ident, ItemEnum, LitStr, Result, Type, parse_quote,
    spanned::Spanned,
};

const MAPPINGS: &[(&[&str], u16)] = &[
    (&["not_found"], 404),
    (&["bad_request", "invalid", "validation"], 400),
    (&["unauthorized"], 401),
    (&["forbidden", "access_denied"], 403),
    (&["conflict", "already_exists"], 409),
    (&["timeout"], 408),
    (&["rate_limit", "too_many_requests"], 429),
    (&["not_implemented"], 501),
    (&["unavailable", "service_unavailable"], 503),
    (&["internal"], 500),
];

pub(super) struct ErrorVariantInfo {
    pub ident: Ident,
    pub status: Expr,
    pub code: String,
    pub message: String,
    pub source: Option<Type>,
    pub help: Option<String>,
    pub cfg_attrs: Vec<Attribute>,
    pub transparent: bool,
    pub extra_from: Vec<Type>,
}

pub(super) fn parse_and_clean_variants(item: &mut ItemEnum) -> Result<Vec<ErrorVariantInfo>> {
    let mut infos = Vec::new();

    for variant in &mut item.variants {
        let ident_str = variant.ident.to_string();
        let cfg_attrs: Vec<_> =
            variant.attrs.iter().filter(|a| a.path().is_ident("cfg")).cloned().collect();

        let initial_status = guess_status_by_ident(&ident_str).unwrap_or_else(|| parse_quote!(500));

        let mut info = ErrorVariantInfo {
            ident: variant.ident.clone(),
            status: initial_status,
            code: ident_str.to_shouty_snake_case(),
            message: to_sentence_case(&ident_str),
            source: None,
            help: None,
            cfg_attrs,
            extra_from: Vec::new(),
            transparent: false,
        };

        let mut i = 0;
        while i < variant.attrs.len() {
            let path = variant.attrs[i].path();
            if path.is_ident("error") || path.is_ident("transparent") {
                let attr = variant.attrs.remove(i);
                process_custom_attribute(&attr, &mut info)?;
            } else {
                i += 1;
            }
        }

        if info.transparent && info.source.is_none() {
            if let Fields::Unnamed(f) = &variant.fields
                && f.unnamed.len() == 1
            {
                info.source = Some(f.unnamed[0].ty.clone());
            }

            if info.source.is_none() {
                return Err(Error::new(
                    variant.span(),
                    "Transparent variant must have exactly one field or an explicit 'source' type",
                ));
            }
        }

        infos.push(info);
    }

    Ok(infos)
}

fn process_custom_attribute(attr: &Attribute, info: &mut ErrorVariantInfo) -> Result<()> {
    if attr.path().is_ident("transparent") {
        info.status = parse_quote!(0);
        info.code = String::new();
        info.message = String::new();
        info.source = None;
        info.help = None;
        info.transparent = true;
    }

    match &attr.meta {
        syn::Meta::Path(_) => Ok(()),
        syn::Meta::List(list) => {
            if let Ok(expr) = list.parse_args::<Expr>()
                && !matches!(expr, Expr::Assign(_))
            {
                if info.transparent {
                    let type_str = quote::quote!(#expr).to_string();
                    info.source = Some(syn::parse_str(&type_str)?);
                } else {
                    info.status = expr;
                }
                return Ok(());
            }
            attr.parse_nested_meta(|meta| attr_mutator(&meta, info))
        },
        syn::Meta::NameValue(_) => {
            Err(Error::new_spanned(attr, "Expected list format: #[error(...)]"))
        },
    }
}

fn attr_mutator(meta: &syn::meta::ParseNestedMeta<'_>, info: &mut ErrorVariantInfo) -> Result<()> {
    let path = &meta.path;

    if path.is_ident("status") || path.is_ident("kind") {
        info.status = meta.value()?.parse()?;
    } else if path.is_ident("message") || path.is_ident("msg") {
        info.message = meta.value()?.parse::<LitStr>()?.value();
    } else if path.is_ident("code") {
        info.code = meta.value()?.parse::<LitStr>()?.value().to_shouty_snake_case();
    } else if path.is_ident("source") || path.is_ident("src") {
        info.source = Some(meta.value()?.parse()?);
    } else if path.is_ident("help") || path.is_ident("hlp") {
        info.help = Some(meta.value()?.parse::<LitStr>()?.value());
    } else if path.is_ident("from") {
        let input = meta.value()?;
        let content;
        syn::bracketed!(content in input);
        let types = content.parse_terminated(Type::parse, syn::Token![,])?;
        info.extra_from.extend(types);
    } else {
        return Err(meta.error(format!("Unknown attribute parameter: {:?}", path.get_ident())));
    }
    Ok(())
}

fn to_sentence_case(s: &str) -> String {
    let spaced = s.to_snake_case().replace('_', " ");
    let mut chars = spaced.chars();
    chars.next().map_or_else(String::new, |f| f.to_uppercase().collect::<String>() + chars.as_str())
}

fn guess_status_by_ident(ident: &str) -> Option<Expr> {
    let ident = ident.to_lowercase();

    for (keywords, code) in MAPPINGS {
        if keywords.iter().any(|&k| ident.contains(k)) {
            return Some(parse_quote!(#code));
        }
    }
    None
}

use dioxus::prelude::*;
pub use nx_i18n::{FluentArgs, I18n, Locale, LocaleError};
use std::ops::Deref;
use std::sync::OnceLock;

static LOCALE: GlobalSignal<Locale> = GlobalSignal::new(Locale::system);
static I18N: OnceLock<I18n> = OnceLock::new();

/// Returns the currently active application locale.
#[must_use]
pub fn locale() -> impl Deref<Target = Locale> {
    LOCALE.read()
}

/// Sets the currently active application locale.
///
/// # Errors
///
/// Returns [`LocaleError`] if the provided locale string is invalid.
pub fn set_locale(value: impl AsRef<str>) -> Result<(), LocaleError> {
    let locale = Locale::parse(value)?;
    LOCALE.with_mut(|current| *current = locale);
    Ok(())
}

/// Sets the currently active application locale.
pub fn set_locale_parsed(locale: Locale) {
    LOCALE.with_mut(|current| *current = locale);
}

/// Returns a shared i18n runtime instance.
#[must_use]
pub fn i18n() -> &'static I18n {
    I18N.get_or_init(I18n::new)
}

/// Translates a key using the current application locale.
#[must_use]
pub fn translate(key: impl AsRef<str>) -> String {
    translate_with_args(key, None)
}

/// Translates a key using the current application locale and optional Fluent arguments.
#[must_use]
pub fn translate_with_args(key: impl AsRef<str>, args: Option<&FluentArgs<'_>>) -> String {
    let i18n = i18n();
    let locale = &*locale();
    i18n.translate(locale, key.as_ref(), args).into_owned()
}

#[doc(hidden)]
#[must_use]
pub fn translate_with_named_args(key: impl AsRef<str>, args: &FluentArgs<'_>) -> String {
    translate_with_args(key, Some(args))
}

#[macro_export]
macro_rules! t {
    ($key:expr) => {{
        $crate::macros::i18n::translate($key)
    }};

    ($key:expr, $($name:ident : $value:expr),+ $(,)?) => {{
        let mut args = $crate::macros::i18n::FluentArgs::new();
        $(
            args.set(stringify!($name), $value);
        )+
        $crate::macros::i18n::translate_with_named_args($key, &args)
    }};

    ($key:expr, ) => {{
        $crate::macros::i18n::translate($key)
    }};
}

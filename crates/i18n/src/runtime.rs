use crate::bundle::{
    available_languages, available_locales, format_message, has_language, translation_candidates,
};
use crate::locale::Locale;
use fluent::FluentArgs;
use std::borrow::Cow;
use tracing::{debug, warn};

/// A lightweight localization facade.
///
/// This type is intentionally zero-sized:
/// all loaded bundles are stored in a global cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct I18n;

impl I18n {
    /// Constructs a new i18n facade.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Translates a key using the provided locale string.
    ///
    /// This is a convenience wrapper. For hot paths, prefer [`Self::translate`]
    /// with a pre-parsed [`Locale`].
    ///
    /// # Errors
    ///
    /// This method does not return an error. If the locale string is invalid,
    /// the key itself is returned and the issue is logged with `warn!`.
    #[must_use]
    pub fn t(&self, lang: impl AsRef<str>, key: impl AsRef<str>) -> Cow<'static, str> {
        self.tr(lang, key, None)
    }

    /// Translates a key using the provided locale string and optional Fluent arguments.
    ///
    /// This is a convenience wrapper. For hot paths, prefer [`Self::translate`]
    /// with a pre-parsed [`Locale`].
    ///
    /// # Errors
    ///
    /// This method does not return an error. If the locale string is invalid,
    /// the key itself is returned and the issue is logged with `warn!`.
    #[must_use]
    pub fn tr(
        &self,
        lang: impl AsRef<str>,
        key: impl AsRef<str>,
        args: Option<&FluentArgs<'_>>,
    ) -> Cow<'static, str> {
        let key = key.as_ref();
        let locale = match Locale::parse(lang) {
            Ok(locale) => locale,
            Err(err) => {
                warn!("Invalid locale input for key `{key}`: {err}");
                return Cow::Owned(key.to_owned());
            },
        };

        self.translate(&locale, key, args)
    }

    /// Fast-path translation using a pre-parsed locale.
    ///
    /// # Errors
    ///
    /// This method does not return an error. If the translation key is missing
    /// for the requested locale and fallback languages, the key itself is returned.
    /// Formatting issues inside an existing Fluent message are logged, and the
    /// formatted value is still returned.
    #[must_use]
    pub fn translate(
        &self,
        locale: &Locale,
        key: &str,
        args: Option<&FluentArgs<'_>>,
    ) -> Cow<'static, str> {
        for lang in translation_candidates(locale) {
            if let Some(value) = format_message(lang, key, args) {
                if lang != locale.as_str() {
                    debug!(
                        "Translation fallback used for `{key}`: `{}` -> `{lang}`",
                        locale.as_str()
                    );
                }
                return value;
            }
        }

        warn!("Translation missing for key `{key}` [{}]", locale.as_str());
        Cow::Owned(key.to_owned())
    }

    /// Translates a key using a pre-parsed locale reference.
    ///
    /// This is identical to [`Self::translate`], but makes the fast-path intent explicit.
    ///
    /// # Errors
    ///
    /// This method does not return an error. If the translation key is missing
    /// for the requested locale and fallback languages, the key itself is returned.
    #[must_use]
    pub fn tr_locale(
        &self,
        locale: &Locale,
        key: impl AsRef<str>,
        args: Option<&FluentArgs<'_>>,
    ) -> Cow<'static, str> {
        self.translate(locale, key.as_ref(), args)
    }

    /// Returns the list of loaded bundle languages.
    #[must_use]
    pub fn available_languages() -> Vec<&'static str> {
        available_languages()
    }

    /// Returns the list of available locales.
    #[must_use]
    pub fn available_locales() -> Vec<Locale> {
        available_locales()
    }

    /// Returns `true` if a bundle exists for the given language.
    #[must_use]
    pub fn has_language(lang: &str) -> bool {
        has_language(lang)
    }
}

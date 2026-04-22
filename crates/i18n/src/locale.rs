use nx_error::error;
use unic_langid::LanguageIdentifier;

/// A normalized locale identifier used by the i18n runtime.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale(String);

impl Locale {
    /// Creates a locale from user input and normalizes it to a canonical form.
    ///
    /// Examples:
    /// - `EN` -> `en`
    /// - `uk-UA` -> `uk-UA`
    ///
    /// # Errors
    ///
    /// Returns [`LocaleError::Empty`] if the input is empty or contains only
    /// whitespace.
    ///
    /// Returns [`LocaleError::Invalid`] if the input is not a valid language tag
    /// according to `unic-langid`.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, LocaleError> {
        let raw = value.as_ref().trim();
        if raw.is_empty() {
            return Err(LocaleError::empty());
        }

        let normalized = normalize_language_tag(raw)?;
        Ok(Self(normalized))
    }

    /// Returns the normalized locale tag as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the primary language subtag used for fallback.
    #[must_use]
    pub fn primary_language(&self) -> &str {
        self.0.split('-').next().unwrap_or("en")
    }

    /// Returns the system locale or `en` if the system locale is invalid.
    ///
    /// # Panics
    ///
    /// It's safe to unwrap here because the default locale is always valid.
    #[must_use]
    pub fn system() -> Self {
        Self::parse(system_locale())
            .unwrap_or_else(|_| Self::parse("en").expect("default locale must be valid"))
    }
}

impl AsRef<str> for Locale {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Locale parsing/normalization error.
#[error]
pub enum LocaleError {
    #[error(
        message = "Locale is empty",
        code = "I18N_LOCALE_EMPTY",
        help = "Please provide a valid locale string, e.g. `en-US`."
    )]
    Empty,
    #[error(message = "Invalid locale", code = "I18N_LOCALE_INVALID")]
    Invalid,
}

fn normalize_language_tag(value: &str) -> Result<String, LocaleError> {
    let parsed: LanguageIdentifier =
        value.parse().map_err(|_| LocaleError::invalid().with_details_fn(|| value.to_owned()))?;

    Ok(parsed.to_string())
}

pub(crate) fn system_locale() -> String {
    let raw = sys_locale::get_locale()
        .as_deref()
        .map(normalize_locale)
        .filter(|locale| is_supported_locale_format(locale))
        .unwrap_or_else(|| "en-US".to_owned());

    if crate::bundle::has_language(&raw) {
        return raw;
    }

    let primary = raw.split('-').next().unwrap_or("en");
    if crate::bundle::has_language(primary) {
        return primary.to_owned();
    }

    "en".to_owned()
}

fn normalize_locale(locale: &str) -> String {
    locale.split('.').next().unwrap_or(locale).replace('_', "-")
}

fn is_supported_locale_format(locale: &str) -> bool {
    !locale.is_empty() && !matches!(locale, "C" | "POSIX")
}

#![allow(unused_crate_dependencies)]

use nx_i18n::{I18n, Locale, tr};

#[test]
fn parses_and_normalizes_locale() {
    let locale = Locale::parse("EN").expect("locale should parse");
    assert_eq!(locale.as_str(), "en");
    assert_eq!(locale.primary_language(), "en");
}

#[test]
fn parses_region_locale_and_extracts_primary_language() {
    let locale = Locale::parse("uk-UA").expect("locale should parse");
    assert_eq!(locale.as_str(), "uk-UA");
    assert_eq!(locale.primary_language(), "uk");
}

#[test]
fn translates_locale_empty_message_in_default_language() {
    let i18n = I18n::new();

    let text = i18n.t("en", tr::i18n::I18N_LOCALE_EMPTY);

    assert_eq!(text, "The language code is empty. Please select a valid language.");
}

#[test]
fn translates_locale_invalid_message_in_default_language() {
    let i18n = I18n::new();

    let text = i18n.t("en", tr::i18n::I18N_LOCALE_INVALID);

    assert_eq!(text, "The selected language is not supported or the format is invalid.");
}

#[test]
fn exposes_available_languages() {
    let langs = I18n::available_languages();

    assert!(langs.contains(&"en"));
    assert!(langs.contains(&"uk"));
}

#[test]
fn reports_existing_language_bundle() {
    assert!(I18n::has_language("en"));
    assert!(I18n::has_language("uk"));
}

#[test]
fn translates_existing_key_in_default_language() {
    let i18n = I18n::new();

    let text = i18n.t("en", tr::i18n::I18N_LOCALE_EMPTY);

    assert_eq!(text, "The language code is empty. Please select a valid language.");
}

#[test]
fn translates_existing_key_in_uk_language() {
    let i18n = I18n::new();

    let text = i18n.t("uk", tr::i18n::I18N_LOCALE_EMPTY);

    assert_eq!(text, "The language code is empty. Please select a valid language.");
}

#[test]
fn falls_back_to_primary_language_for_region_locale() {
    let i18n = I18n::new();
    let locale = Locale::parse("uk-UA").expect("locale should parse");

    let text = i18n.translate(&locale, tr::i18n::I18N_LOCALE_EMPTY, None);

    assert_eq!(text, "The language code is empty. Please select a valid language.");
}

#[test]
fn falls_back_to_default_language_when_requested_language_is_missing() {
    let i18n = I18n::new();
    let locale = Locale::parse("pl").expect("locale should parse");

    let text = i18n.translate(&locale, tr::i18n::I18N_LOCALE_EMPTY, None);

    assert_eq!(text, "The language code is empty. Please select a valid language.");
}

#[test]
fn returns_key_itself_when_translation_is_missing_everywhere() {
    let i18n = I18n::new();
    let locale = Locale::parse("en").expect("locale should parse");
    let key = "this-key-does-not-exist";

    let text = i18n.translate(&locale, key, None);

    assert_eq!(text, key);
}

#[test]
fn convenience_api_matches_preparsed_locale_api() {
    let i18n = I18n::new();
    let locale = Locale::parse("uk-UA").expect("locale should parse");
    let key = tr::i18n::I18N_LOCALE_EMPTY;

    let via_string = i18n.tr("uk-UA", key, None);
    let via_locale = i18n.tr_locale(&locale, key, None);

    assert_eq!(via_string, via_locale);
}

#[test]
fn generated_keys_are_accessible() {
    assert_eq!(tr::i18n::I18N_LOCALE_EMPTY, "i18n-locale-empty");
    assert_eq!(tr::i18n::I18N_LOCALE_INVALID, "i18n-locale-invalid");
}

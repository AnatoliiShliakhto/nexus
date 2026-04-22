use crate::{Locale, generated};
use fluent::{FluentArgs, FluentBundle, FluentResource};
use fxhash::FxHashMap;
use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::LazyLock;
use tracing::error;
use unic_langid::LanguageIdentifier;

pub(crate) const DEFAULT_LANGUAGE: &str = "en";

thread_local! {
    static BUNDLES_MAP: RefCell<Option<FxHashMap<&'static str, FluentBundle<FluentResource>>>> =
        const { RefCell::new(None) };
}

pub(crate) static AVAILABLE_LANGUAGES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let mut langs = generated::BUNDLES.iter().map(|(lang, _)| *lang).collect::<Vec<_>>();
    langs.sort_unstable();
    langs
});

#[must_use]
pub(crate) fn available_languages() -> Vec<&'static str> {
    AVAILABLE_LANGUAGES.clone()
}

#[must_use]
pub(crate) fn available_locales() -> Vec<Locale> {
    AVAILABLE_LANGUAGES.iter().copied().filter_map(|lang| Locale::parse(lang).ok()).collect()
}

#[must_use]
pub(crate) fn has_language(lang: &str) -> bool {
    generated::BUNDLES.iter().any(|(candidate, _)| *candidate == lang)
}

pub(crate) fn format_message(
    lang: &str,
    key: &str,
    args: Option<&FluentArgs<'_>>,
) -> Option<Cow<'static, str>> {
    with_bundles(|bundles| {
        let bundle = bundles.get(lang)?;
        let msg = bundle.get_message(key)?;
        let pattern = msg.value()?;

        let mut errors = Vec::with_capacity(2);
        let result = bundle.format_pattern(pattern, args, &mut errors);

        if cfg!(debug_assertions) && !errors.is_empty() {
            error!("Fluent formatting errors for `{key}` [{lang}]: {errors:?}");
        }

        Some(Cow::Owned(result.into_owned()))
    })
}

pub(crate) fn translation_candidates(locale: &Locale) -> TranslationCandidates<'_> {
    TranslationCandidates::new(locale)
}

pub(crate) struct TranslationCandidates<'a> {
    locale: &'a str,
    primary: &'a str,
    state: CandidateState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateState {
    Locale,
    Primary,
    Default,
    Done,
}

impl<'a> TranslationCandidates<'a> {
    fn new(locale: &'a Locale) -> Self {
        Self {
            locale: locale.as_str(),
            primary: locale.primary_language(),
            state: CandidateState::Locale,
        }
    }
}

impl<'a> Iterator for TranslationCandidates<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            CandidateState::Locale => {
                self.state = if self.primary != self.locale {
                    CandidateState::Primary
                } else if self.locale != DEFAULT_LANGUAGE {
                    CandidateState::Default
                } else {
                    CandidateState::Done
                };
                Some(self.locale)
            },
            CandidateState::Primary => {
                self.state = if self.locale == DEFAULT_LANGUAGE {
                    CandidateState::Done
                } else {
                    CandidateState::Default
                };
                Some(self.primary)
            },
            CandidateState::Default => {
                self.state = CandidateState::Done;
                Some(DEFAULT_LANGUAGE)
            },
            CandidateState::Done => None,
        }
    }
}

fn with_bundles<R>(
    f: impl FnOnce(&FxHashMap<&'static str, FluentBundle<FluentResource>>) -> R,
) -> R {
    BUNDLES_MAP.with(|cell| {
        if cell.borrow().is_none() {
            let bundles = load_bundles();
            *cell.borrow_mut() = Some(bundles);
        }

        let borrow = cell.borrow();
        f(borrow.as_ref().expect("bundles must be initialized"))
    })
}

fn load_bundles() -> FxHashMap<&'static str, FluentBundle<FluentResource>> {
    let mut map = FxHashMap::default();

    for (lang, source) in generated::BUNDLES {
        let res = FluentResource::try_new(source.to_string()).unwrap_or_else(|(res, errs)| {
            error!("Failed to parse FTL for `{lang}`: {errs:?}");
            res
        });

        let Ok(lang_id) = lang.parse::<LanguageIdentifier>() else {
            error!("Invalid language id in generated bundles: `{lang}`");
            continue;
        };

        let mut bundle = FluentBundle::new(vec![lang_id]);

        if let Err(errs) = bundle.add_resource(res) {
            error!("Conflicting resources in `{lang}`: {errs:?}");
        }

        map.insert(*lang, bundle);
    }

    map
}

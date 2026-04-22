//! Nexus i18n runtime.
//!
//! This crate provides a lightweight Fluent-based localization engine with:
//! * locale parsing and normalization
//! * cached bundle lookup
//! * fallback support
//! * generated type-safe translation keys
//!
//! # Performance
//!
//! Parse a locale once and reuse it in hot paths:
//!
//! ```rust
//! let locale = nx_i18n::Locale::parse("uk-UA")?;
//! let text = i18n.translate(&locale, nx_i18n::tr::database::DATABASE_AUTH_ERROR, None);
//! ```

mod bundle;
mod generated;
mod locale;
mod runtime;

pub use fluent::FluentArgs;
pub use generated::localization as tr;
pub use locale::{Locale, LocaleError};
pub use runtime::I18n;

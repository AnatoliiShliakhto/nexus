use crate::error::ErrorExt;
use nx_error::prelude::*;
use spin_sdk::variables;
use std::borrow::Cow;
use url::{ParseError, Url};

#[error]
pub enum SpinEnvironmentError {
    #[error(
        message = "Required configuration variable is missing",
        status = ErrorStatus::InternalServerError,
        code = "HTTP_SERVICE_UNCONFIGURED"
    )]
    VariableNotSet,

    #[error(
        message = "Configuration variable contains an invalid URL",
        status = ErrorStatus::InternalServerError,
        code = "HTTP_SERVICE_CONFIGURATION_INVALID",
        source = ParseError,
    )]
    UrlParse,
}

/// Retrieves a `spin` variable by name with an optional static fallback.
///
/// This function implements an allocation-efficient retrieval strategy:
/// 1. If the variable exists in the Spin environment, it returns an owned string.
/// 2. If missing, it attempts to use the provided `default` static string.
/// 3. If both are missing, it returns a structured `SpinVariableError`.
///
/// # Returns
/// * `Ok(Cow::Owned)` - Variable found in environment.
/// * `Ok(Cow::Borrowed)` - Using the provided `default` fallback.
/// * `Err(SpinVariableError)` - Variable missing and no default provided.
///
/// # Errors
/// * [`SpinVariableError::VariableNotSet`] - If the key is missing and no default is provided.
pub async fn get_spin_var(
    name: &str,
    default: Option<&'static str>,
) -> Result<Cow<'static, str>, SpinEnvironmentError> {
    variables::get(name)
        .await
        .map(Cow::Owned)
        .or_else(|_| {
            default.map(Cow::Borrowed).ok_or_else(|| {
                SpinEnvironmentError::variable_not_set()
                    .with_details(format!("Variable: {name}"))
                    .with_help(format!(
                        "Set the `spin` variable `{name}` to configure the service."
                    ))
            })
        })
        .map_err(|e| {
            e.with_details(format!("Variable: {name}"))
                .with_help(format!("Set the `spin` variable `{name}` to configure the service."))
        })
}

/// Retrieves and normalizes an URL from the `spin` environment.
///
/// This function fetches the configuration, strips any trailing slashes to prevent
/// path duplication during downstream `join()` operations, and parses it into a [`Url`].
///
/// # Normalization
/// To maintain consistency in service-to-service communication, the resulting URL
/// is guaranteed to have no trailing slash (e.g., `http://api/` becomes `http://api`).
///
/// # Errors
/// * [`SpinVariableError::VariableNotSet`] - If the key is missing and no default is provided.
/// * [`SpinVariableError::UrlParse`] - If the retrieved value is not a valid URL.
///
/// # Examples
/// ```rust,ignore
/// // Environment: gateway_url = "http://api.internal/"
/// let url = get_spin_url("gateway_url", None)?;
/// assert_eq!(url.as_str(), "http://api.internal");
/// ```
pub async fn get_spin_url(
    name: &str,
    default: Option<&'static str>,
) -> Result<Url, SpinEnvironmentError> {
    let url_str = get_spin_var(name, default).await?;

    Url::parse(url_str.trim_end_matches('/')).map_err(|e| {
        SpinEnvironmentError::from(e)
            .with_details(format!("Variable: {name}"))
            .with_help(format!("Check if `{name}` is a valid URL (e.g., http://target.internal)"))
    })
}

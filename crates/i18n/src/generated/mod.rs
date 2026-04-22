pub(crate) const BUNDLES: &[(&str, &str)] = &[
    ("en", include_str!("i18n/en.ftl")),
    ("uk", include_str!("i18n/uk.ftl")),
];

pub mod localization {
    pub mod access {
        pub const ACCESS_INTERNAL_ERROR: &str = "access-internal-error";
    }
    pub mod account {
        pub const ACCOUNT_INTERNAL_ERROR: &str = "account-internal-error";
    }
    pub mod audit {
        pub const AUDIT_INTERNAL_ERROR: &str = "audit-internal-error";
    }
    pub mod auth {
        pub const AUTH_INTERNAL_ERROR: &str = "auth-internal-error";
        pub const AUTH_RESOURCE_NOT_FOUND: &str = "auth-resource-not-found";
    }
    pub mod config {
        pub const CONFIG_ENV_MISSING: &str = "config-env-missing";
        pub const CONFIG_INVALID: &str = "config-invalid";
        pub const CONFIG_LOAD_FAILURE: &str = "config-load-failure";
        pub const CONFIG_NOT_FOUND: &str = "config-not-found";
        pub const CONFIG_VAULT_BUILDER_ERROR: &str = "config-vault-builder-error";
        pub const CONFIG_VAULT_ERROR: &str = "config-vault-error";
    }
    pub mod console {
        pub const CON_CONFIGURATION_ERROR: &str = "con-configuration-error";
        pub const CON_INTERNAL_ERROR: &str = "con-internal-error";
        pub const CON_LOGIN_DESCRIPTION: &str = "con-login-description";
        pub const CON_LOGIN_FOOTER: &str = "con-login-footer";
        pub const CON_LOGIN_FORGOT_ACCESS_KEY: &str = "con-login-forgot-access-key";
        pub const CON_LOGIN_PASSWORD_LABEL: &str = "con-login-password-label";
        pub const CON_LOGIN_SIGNIN: &str = "con-login-signin";
        pub const CON_LOGIN_TITLE: &str = "con-login-title";
        pub const CON_LOGIN_USERNAME_LABEL: &str = "con-login-username-label";
        pub const CON_LOGIN_USERNAME_PLACEHOLDER: &str = "con-login-username-placeholder";
    }
    pub mod database {
        pub const DATABASE_AUTH_ERROR: &str = "database-auth-error";
        pub const DATABASE_CONFIG_INVALID: &str = "database-config-invalid";
        pub const DATABASE_DATA_INVALID: &str = "database-data-invalid";
        pub const DATABASE_INTERNAL_ERROR: &str = "database-internal-error";
        pub const DATABASE_QUERY_EXECUTION_FAILED: &str = "database-query-execution-failed";
        pub const DATABASE_RESOURCE_NOT_FOUND: &str = "database-resource-not-found";
        pub const DATABASE_UNSUPPORTED_CONTENT_TYPE: &str = "database-unsupported-content-type";
    }
    pub mod http {
        pub const HTTP_REQUEST_BUILD_FAILURE: &str = "http-request-build-failure";
        pub const PROXY_BUILD_FAILED: &str = "proxy-build-failed";
        pub const PROXY_TARGET_INVALID: &str = "proxy-target-invalid";
        pub const PROXY_TARGET_MISSING: &str = "proxy-target-missing";
        pub const SERVICE_CONFIGURATION_INVALID: &str = "service-configuration-invalid";
        pub const SERVICE_UNCONFIGURED: &str = "service-unconfigured";
        pub const UPSTREAM_UNAVAILABLE: &str = "upstream-unavailable";
        pub const URL_INVALID: &str = "url-invalid";
    }
    pub mod i18n {
        pub const I18N_LOCALE_EMPTY: &str = "i18n-locale-empty";
        pub const I18N_LOCALE_INVALID: &str = "i18n-locale-invalid";
    }
    pub mod organization {
        pub const ORG_INTERNAL_ERROR: &str = "org-internal-error";
    }
    pub mod ui {
        pub const UI_NOTIFICATIONS_CLEAR: &str = "ui-notifications-clear";
        pub const UI_NOTIFICATIONS_EMPTY: &str = "ui-notifications-empty";
        pub const UI_NOTIFICATIONS_HEADER: &str = "ui-notifications-header";
    }
}

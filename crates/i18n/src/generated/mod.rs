pub(crate) const BUNDLES: &[(&str, &str)] =
    &[("en", include_str!("i18n/en.ftl")), ("uk", include_str!("i18n/uk.ftl"))];

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
        pub const CON_ERROR_TITLE: &str = "con-error-title";
        pub const CON_INTERNAL_ERROR: &str = "con-internal-error";
        pub const CON_INVALID_CREDENTIALS: &str = "con-invalid-credentials";
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
    pub mod gateway {
        pub const GW_AUTH_ACCESS_DENIED: &str = "gw-auth-access-denied";
        pub const GW_INFRA_REDIS_ERROR: &str = "gw-infra-redis-error";
        pub const GW_PAYLOAD_INVALID_FORMAT: &str = "gw-payload-invalid-format";
        pub const GW_PROXY_DISPATCH_FAILED: &str = "gw-proxy-dispatch-failed";
        pub const GW_PROXY_REQUEST_BUILD_FAILED: &str = "gw-proxy-request-build-failed";
        pub const GW_PROXY_TARGET_INVALID: &str = "gw-proxy-target-invalid";
        pub const GW_PROXY_TARGET_MISSING: &str = "gw-proxy-target-missing";
        pub const GW_PROXY_URI_PARSE_FAILED: &str = "gw-proxy-uri-parse-failed";
        pub const GW_SYS_INTERNAL_ERROR: &str = "gw-sys-internal-error";
        pub const GW_SYS_SHUTDOWN_INIT_FAILED: &str = "gw-sys-shutdown-init-failed";
        pub const GW_TRAFFIC_OVERLOADED: &str = "gw-traffic-overloaded";
        pub const GW_TRAFFIC_UPSTREAM_TIMEOUT: &str = "gw-traffic-upstream-timeout";
        pub const IDENTITY_ACCOUNT_BLOCKED: &str = "identity-account-blocked";
        pub const IDENTITY_AUTHORIZATION_MISSING: &str = "identity-authorization-missing";
        pub const IDENTITY_DPOP_ATH_MISMATCH: &str = "identity-dpop-ath-mismatch";
        pub const IDENTITY_DPOP_EXPIRED: &str = "identity-dpop-expired";
        pub const IDENTITY_DPOP_INVALID: &str = "identity-dpop-invalid";
        pub const IDENTITY_DPOP_MISSING: &str = "identity-dpop-missing";
        pub const IDENTITY_DPOP_NONCE_REQUIRED: &str = "identity-dpop-nonce-required";
        pub const IDENTITY_DPOP_REPLAY_DETECTED: &str = "identity-dpop-replay-detected";
        pub const IDENTITY_DPOP_UNSUPPORTED_KEY: &str = "identity-dpop-unsupported-key";
        pub const IDENTITY_DPOP_VERIFICATION_FAILED: &str = "identity-dpop-verification-failed";
        pub const IDENTITY_INTERNAL_ERROR: &str = "identity-internal-error";
        pub const IDENTITY_INVALID_CREDENTIALS: &str = "identity-invalid-credentials";
        pub const IDENTITY_IP_RESTRICTED: &str = "identity-ip-restricted";
        pub const IDENTITY_REFRESH_TOKEN_MISSING: &str = "identity-refresh-token-missing";
        pub const IDENTITY_SESSION_DATA_CORRUPTION: &str = "identity-session-data-corruption";
        pub const IDENTITY_SESSION_UNAUTHORIZED: &str = "identity-session-unauthorized";
    }
    pub mod http {
        pub const HTTP_PROXY_BUILD_FAILED: &str = "http-proxy-build-failed";
        pub const HTTP_PROXY_TARGET_INVALID: &str = "http-proxy-target-invalid";
        pub const HTTP_PROXY_TARGET_MISSING: &str = "http-proxy-target-missing";
        pub const HTTP_REQUEST_BUILD_FAILURE: &str = "http-request-build-failure";
        pub const HTTP_SERVICE_CONFIGURATION_INVALID: &str = "http-service-configuration-invalid";
        pub const HTTP_SERVICE_UNCONFIGURED: &str = "http-service-unconfigured";
        pub const HTTP_UPSTREAM_UNAVAILABLE: &str = "http-upstream-unavailable";
        pub const HTTP_URL_INVALID: &str = "http-url-invalid";
    }
    pub mod i18n {
        pub const I18N_LOCALE_EMPTY: &str = "i18n-locale-empty";
        pub const I18N_LOCALE_INVALID: &str = "i18n-locale-invalid";
    }
    pub mod lockbox {
        pub const LOCKBOX_CORE_ERROR: &str = "lockbox-core-error";
        pub const LOCKBOX_INTERNAL_ERROR: &str = "lockbox-internal-error";
        pub const LOCKBOX_INVALID_STORE: &str = "lockbox-invalid-store";
        pub const LOCKBOX_NO_STORE: &str = "lockbox-no-store";
        pub const LOCKBOX_PLATFORM_NOT_SUPPORTED: &str = "lockbox-platform-not-supported";
        pub const LOCKBOX_SERIALIZATION_ERROR: &str = "lockbox-serialization-error";
    }
    pub mod organization {
        pub const ORG_INTERNAL_ERROR: &str = "org-internal-error";
    }
    pub mod ui {
        pub const UI_NOTIFICATIONS_CLEAR: &str = "ui-notifications-clear";
        pub const UI_NOTIFICATIONS_EMPTY: &str = "ui-notifications-empty";
        pub const UI_NOTIFICATIONS_HEADER: &str = "ui-notifications-header";
    }
    pub mod web_client {
        pub const WEB_CLIENT_API_ERROR: &str = "web-client-api-error";
        pub const WEB_CLIENT_CONNECTION_FAILED: &str = "web-client-connection-failed";
        pub const WEB_CLIENT_DECODE_ERROR: &str = "web-client-decode-error";
        pub const WEB_CLIENT_DPOP_SIGNING_FAILED: &str = "web-client-dpop-signing-failed";
        pub const WEB_CLIENT_HEADER_ERROR: &str = "web-client-header-error";
        pub const WEB_CLIENT_INTERNAL_ERROR: &str = "web-client-internal-error";
        pub const WEB_CLIENT_REQUEST_FAILED: &str = "web-client-request-failed";
        pub const WEB_CLIENT_REQUEST_TIMEOUT: &str = "web-client-request-timeout";
        pub const WEB_CLIENT_SERIALIZATION_ERROR: &str = "web-client-serialization-error";
    }
}

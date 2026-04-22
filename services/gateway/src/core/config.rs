use nx_config::{Config, Initialized, WithConfig};
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr};

pub(crate) type GatewayConfig = Config<Settings, Initialized, WithConfig>;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub traffic: TrafficConfig,
    pub http_client: HttpClientConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
}

/// HTTP server configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct ServerConfig {
    pub address: IpAddr,
    pub port: u16,
    pub ssl: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { address: IpAddr::V4(Ipv4Addr::UNSPECIFIED), port: 8080, ssl: false }
    }
}

/// Security configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct SecurityConfig {
    pub identity: IdentityConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self { identity: Default::default() }
    }
}

/// Traffic configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct TrafficConfig {
    pub buffer_size: usize,
    pub concurrency_limit: usize,
    pub rate_limit_requests: u64,
    pub rate_limit_duration_secs: u64,
}

impl Default for TrafficConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            concurrency_limit: 100,
            rate_limit_requests: 500,
            rate_limit_duration_secs: 1,
        }
    }
}

/// HTTP-client settings
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct HttpClientConfig {
    pub pool_max_idle_per_host: usize,
    pub pool_idle_timeout_secs: u64,
    pub connect_timeout_millis: u64,
    pub tcp_keepalive_secs: u64,
    pub tcp_nodelay: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            pool_max_idle_per_host: 32,
            pool_idle_timeout_secs: 30,
            connect_timeout_millis: 500,
            tcp_keepalive_secs: 90,
            tcp_nodelay: true,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct IdentityConfig {
    pub dpop: DpopConfig,
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct DpopConfig {
    /// Storage backend: "redis" or "in-memory"
    pub provider: DpopStorageProvider,
    /// Maximum items in the local cache to prevent OOM
    pub max_capacity: u64,
    /// Acceptable time drift for DPoP timestamps (seconds)
    pub time_window_sec: i64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DpopStorageProvider {
    Redis,
    InMemory,
}

impl Default for DpopConfig {
    fn default() -> Self {
        Self {
            provider: DpopStorageProvider::InMemory,
            max_capacity: 100_000,
            time_window_sec: 120,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RedisConfig {
    pub url: String,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self { url: String::new() }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub(crate) struct DatabaseConfig {
    pub url: String,
    pub namespace: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct SessionConfig {
    /// Maximum sessions to keep in the in-memory cache
    pub cache_capacity: u64,
    /// Access token TTL (seconds)
    pub access_token_ttl_sec: u64,
    /// Refresh token TTL (days)
    pub refresh_token_ttl_days: u64,
    /// Length of the refresh token in bytes
    pub refresh_token_len: u8,
    /// Grace period for refreshing session tokens (seconds)
    pub grace_period_sec: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cache_capacity: 1000,
            access_token_ttl_sec: 900,  // 15 mins
            refresh_token_ttl_days: 30, // 30 days
            refresh_token_len: 64,
            grace_period_sec: 30,
        }
    }
}

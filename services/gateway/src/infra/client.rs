use crate::core::config::HttpClientConfig;
use axum::body::Body;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::{TokioExecutor, TokioTimer};
use std::time::Duration;

pub(crate) type GatewayClient = Client<HttpConnector, Body>;

pub(crate) fn create_client(config: &HttpClientConfig) -> GatewayClient {
    let mut connector = HttpConnector::new();

    connector.set_nodelay(config.tcp_nodelay);
    connector.set_keepalive(Some(Duration::from_secs(config.tcp_keepalive_secs)));
    connector.set_connect_timeout(Some(Duration::from_millis(config.connect_timeout_millis)));
    Client::builder(TokioExecutor::new())
        .pool_timer(TokioTimer::new())
        .pool_max_idle_per_host(config.pool_max_idle_per_host)
        .pool_idle_timeout(Duration::from_secs(config.pool_idle_timeout_secs))
        .build(connector)
}

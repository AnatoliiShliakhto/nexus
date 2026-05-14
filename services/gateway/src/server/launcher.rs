use super::handlers;
use super::router::app;
use super::shutdown::shutdown_handle;
use crate::core::config::Settings;
use crate::error::GatewayError;
use nx_config::Config;
use std::net::SocketAddr;

pub async fn serve() -> Result<(), GatewayError> {
    let _ = *handlers::health::START_TIME;

    let config = Config::from_file::<Settings>("gateway.config").await?;
    let settings = config.get();

    let handle = shutdown_handle();
    let addr = SocketAddr::new(settings.server.address, settings.server.port);
    let app = app(config.clone()).await?;

    tracing::info!(
        "Starting server on {}://{addr:?}",
        if settings.server.ssl { "https" } else { "http" }
    );

    axum_server::bind(addr)
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .map_err(|e| {
            GatewayError::internal().with_details(e.to_string()).with_help("Failed to start server")
        })?;

    Ok(())
}

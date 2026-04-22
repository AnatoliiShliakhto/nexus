use crate::core::error::GatewayError;
use axum_server::Handle;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::signal;
use tracing::{error, info};

pub(crate) fn shutdown_handle() -> Handle<SocketAddr> {
    let handle = Handle::<SocketAddr>::new();
    let shutdown_handle = handle.clone();

    tokio::spawn(async move {
        if let Err(e) = shutdown_signal().await {
            error!("Shutdown signal listener failed: {e:?}");
            return;
        }
        info!("Shutdown signal received, starting graceful shutdown...");
        shutdown_handle.graceful_shutdown(Some(Duration::from_secs(30)));
    });

    handle
}

async fn shutdown_signal() -> Result<(), GatewayError> {
    let ctrl_c = signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .map_err(|e| GatewayError::signal_init_failed().with_details(e.to_string()))?
            .recv()
            .await;
        Ok(())
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<Result<(), GatewayError>>();

    tokio::select! {
        _ = ctrl_c => info!("Terminate signal received"),
        res = terminate => return res,
    }
    Ok(())
}

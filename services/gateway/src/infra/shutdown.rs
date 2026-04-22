use crate::error::GatewayError;
use tokio::signal;
use tracing::info;

async fn shutdown_signal() -> Result<(), GatewayError> {
    let ctrl_c = signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .map_err(|e| GatewayError::shutdown().with_details(e.to_string()))?
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

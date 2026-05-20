#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {{ name | snake_case }}::error::{{ shortname | pascal_case }}Error;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};
use nx_error::ErrorMetadataExt;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let mut layers = Vec::new();

    #[cfg(all(feature = "profiling", tokio_unstable))]
    layers.push(console_subscriber::spawn().boxed());

    let console_layer = layer()
        .compact()
        .with_ansi(true)
        .boxed();
    layers.push(console_layer);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(layers)
        .init();

    if let Err(e) = {{ name | snake_case }}::run().await {
        tracing::error!("\n\n{}", e.report());
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
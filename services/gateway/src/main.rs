#![allow(unused_imports, unused_crate_dependencies)]

use nx_error::ErrorMetadataExt;
use nx_logger::Logger;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let _logger = Logger::builder()
        .name(env!("CARGO_PKG_NAME"))
        .opentelemetry(true)
        .init()
        .expect("Failed to initialize Nexus Logger");

    if let Err(e) = nx_gateway::serve().await {
        tracing::error!("\n\n{}", e.report());
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

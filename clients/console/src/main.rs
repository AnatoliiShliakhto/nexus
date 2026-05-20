#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports, unused_crate_dependencies)]

use nx_console::error::ConsoleError;
use nx_error::ErrorMetadataExt;
use nx_logger::{Logger, Rotation};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nexus")
        .join(env!("CARGO_PKG_NAME"))
        .join("logs");

    let _logger = Logger::builder()
        .name(env!("CARGO_PKG_NAME"))
        .console(true)
        .path(log_dir)
        .rotation(Rotation::DAILY)
        .max_files(30)
        .init()
        .expect("Failed to initialize Nexus Logger");

    if let Err(e) = nx_console::run() {
        tracing::error!("\n\n{}", e.report());
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

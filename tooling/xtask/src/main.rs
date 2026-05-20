#![allow(
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    dead_code
)]

mod domain;
mod error;
mod handlers;
mod services;

use crate::domain::args::{AppCommands, Cli};
use crate::error::AppError;
use clap::Parser;
use nx_error::ErrorMetadataExt;
use nx_logger::Logger;
use std::process::ExitCode;

fn main() -> ExitCode {
    let _logger = Logger::builder()
        .name(env!("CARGO_PKG_NAME"))
        .console(true)
        .init()
        .expect("Failed to initialize Nexus Logger");

    if let Err(e) = run() {
        tracing::error!("\n\n{}", e.report());
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), AppError> {
    dotenvy::dotenv().map_err(|e| AppError::internal().with_details(e.to_string()))?;
    let cli = Cli::parse();

    match cli.command {
        AppCommands::Setup => handlers::setup::setup_project()?,
        AppCommands::Build { project, target, release } => {
            handlers::build::build_project(&project, target.as_ref(), release)?;
        },
        AppCommands::Lint { project, target, release } => {
            handlers::lint::lint_project(&project, target.as_ref(), release)?;
        },
        AppCommands::Dev { action } => handlers::dev::handle_dev_command(action)?,
        AppCommands::Add { name, kind } => handlers::add::add_package(&name, kind)?,

        AppCommands::Codegen { mode } => handlers::codegen::handle_codegen(&mode)?,
        AppCommands::Serve => handlers::spin::run_spin_server()?,
        AppCommands::Keys { action } => handlers::keys::handle_keys(&action)?,
        AppCommands::Vault { action } => handlers::vault::handle_vault(&action)?,
    }

    Ok(())
}

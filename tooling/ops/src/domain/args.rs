//! # CLI Argument Definitions
//!
//! This module defines the command-line interface (CLI) structure using the `clap` crate.
//! It specifies the available subcommands, arguments, and flags for the application.

use clap::{Parser, Subcommand};

/// The main CLI structure parsing command-line arguments.
#[derive(Debug, Parser)]
#[command(bin_name = "migration")]
#[command(author, version, about)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
pub(crate) struct Cli {
    /// The main subcommand to execute.
    #[command(subcommand)]
    pub command: AppCommands,
}

/// Enumeration of available application subcommands.
#[derive(Debug, Subcommand)]
pub(crate) enum AppCommands {
    /// Database migration management
    #[command(subcommand)]
    Migrate(MigrateCommands),

    /// Security and JWT configuration
    #[command(subcommand)]
    Auth(AuthCommands),

    /// Update root (superadmin) login and password
    Admin {
        #[arg(short, long)]
        login: String,
        #[arg(short, long)]
        password: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum MigrateCommands {
    /// Apply pending migrations and sync schema
    Up,
    // /// Roll back the last migration
    // Down,
}

#[derive(Debug, Subcommand)]
pub(crate) enum AuthCommands {
    /// Rotate/Update the JWT public key for database access
    Update,
}

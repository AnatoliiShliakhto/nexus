use crate::handlers::add::CrateKind;
use clap::{Parser, Subcommand};

/// The main CLI structure parsing command-line arguments.
#[derive(Debug, Parser)]
#[command(bin_name = "cargo xtask")]
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
    /// Install required tools (component, audit, nextest, etc.) and add WASM targets.
    Setup,

    /// Build one or more projects within the workspace.
    Build {
        /// Project name or scope (all, clients, components)
        #[arg(
            value_name = "CRATE_NAME|SCOPE",
            default_value = "all",
            help = "The specific project or group to build. Options: all, clients, components, or <CRATE_NAME>"
        )]
        project: String,
        /// The target architecture. Defaults to wasm32-wasip2 for components.
        #[arg(short, long, value_name = "TRIPLE")]
        target: Option<String>,
        /// Build in release mode with optimizations.
        #[arg(short, long)]
        release: bool,
    },

    /// Run lints and formatting checks
    Lint {
        /// Project name or scope (all, apps, components)
        #[arg(default_value = "all")]
        project: String,
        /// The target architecture. Defaults to wasm32-wasip2 for components.
        #[arg(short, long, value_name = "TRIPLE")]
        target: Option<String>,
        /// Run in release mode
        #[arg(short, long)]
        release: bool,
    },

    /// Manage local development infrastructure
    Dev {
        #[command(subcommand)]
        action: DevAction,
    },

    /// Add a new package to the workspace
    Add {
        /// The name of the new project/crate
        name: String,
        /// The type of the crate to create
        #[arg(short, long, value_enum, default_value_t = CrateKind::Component)]
        kind: CrateKind,
    },

    /// Generate workspace artifacts.
    Codegen {
        /// What to generate: all, migrations, i18n
        #[arg(value_name = "MODE", default_value = "all")]
        mode: String,
    },

    /// Serve the spin-server for local development
    Serve,

    /// Manage secret keys
    Keys {
        #[command(subcommand)]
        action: KeysAction,
    },

    /// Manage vault secrets
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
}

/// Enumeration of available development subcommands.
#[derive(Debug, Subcommand)]
pub(crate) enum DevAction {
    /// Start infrastructure services
    Up {},
    /// Stop all services
    Down {
        /// Also remove volumes (wipes the database)
        #[arg(short, long)]
        volumes: bool,
    },
    /// Follow logs from services
    Logs {
        /// Specific service name (e.g., 'surrealdb', 'redis')
        service: Option<String>,
    },
}

/// Enumeration of available keys subcommands.
#[derive(Debug, Subcommand)]
pub(crate) enum KeysAction {
    /// Generate a new key pair for signing JWTs
    Generate,
}

/// Enumeration of available vault subcommands.
#[derive(Debug, Subcommand)]
pub(crate) enum VaultAction {
    /// Update vault keys
    Update,
}

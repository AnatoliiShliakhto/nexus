#![cfg_attr(debug_assertions, allow(unused_imports, dead_code))]

use crate::config::Config;
use crate::domain::args::{AppCommands, AuthCommands, Cli, MigrateCommands};
use crate::error::AppError;
use crate::migration::{AccessRunner, MigrationRunner};
use clap::Parser;
use nx_error::ErrorMetadataExt;
use nx_logger::Logger;
use secrecy::{ExposeSecret, SecretString};
use std::sync::LazyLock;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use tracing::info;

mod config;
mod domain;
mod error;
mod generated;
mod migration;

pub(crate) static DB: LazyLock<Surreal<Any>> = LazyLock::new(Surreal::<Any>::init);

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let _logger = Logger::builder()
        .name(env!("CARGO_PKG_NAME"))
        .console(true)
        .init()
        .expect("Failed to initialize Nexus Logger");

    dotenvy::dotenv().map_err(|e| AppError::internal().with_details(e.to_string()))?;
    let cli = Cli::parse();

    tokio::select! {
        res = run(cli) => {
            if let Err(e) = res {
                tracing::error!("\n\n{}", e.report());
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::warn!("Received Ctrl+C, shutting down gracefully...");
        }
    }

    Ok(())
}

async fn run(cli: Cli) -> Result<(), AppError> {
    let cfg = Config::load()?;
    database_init(&cfg).await?;

    match cli.command {
        AppCommands::Migrate(cmd) => migrate(cmd, &cfg).await?,
        AppCommands::Auth(cmd) => auth(cmd, &cfg).await?,
        AppCommands::Admin { login, password } => update_root(login, password).await?,
    }

    Ok(())
}

async fn migrate(cmd: MigrateCommands, cfg: &Config) -> Result<(), AppError> {
    match cmd {
        MigrateCommands::Up => {
            info!(
                ns = %cfg.namespace.expose_secret(),
                db = %cfg.database.expose_secret(),
                "Applying database migrations...");
            let migration_report = DB.migrate().await?;
            for skipped in migration_report.skipped {
                info!(
                    component = skipped.component,
                    version = skipped.version,
                    "Skipping migration"
                );
            }
            for applied in migration_report.applied {
                info!(
                    component = applied.component,
                    version = applied.version,
                    "Applied migration"
                );
            }
            info!(
                ns = %cfg.namespace.expose_secret(),
                db = %cfg.database.expose_secret(),
                "Database migrations applied successfully"
            );
            update_auth_key(cfg).await?;
        },
    }

    Ok(())
}

async fn auth(cmd: AuthCommands, cfg: &Config) -> Result<(), AppError> {
    match cmd {
        AuthCommands::Update => update_auth_key(cfg).await?,
    }
    Ok(())
}

async fn database_init(cfg: &Config) -> Result<(), AppError> {
    DB.connect(cfg.url.expose_secret()).await?;
    DB.signin(surrealdb::opt::auth::Root {
        username: cfg.user.expose_secret().to_owned(),
        password: cfg.pass.expose_secret().to_owned(),
    })
    .await?;
    DB.use_ns(cfg.namespace.expose_secret()).use_db(cfg.database.expose_secret()).await?;
    Ok(())
}

async fn update_auth_key(cfg: &Config) -> Result<(), AppError> {
    DB.update_pub_key(cfg.public_key.expose_secret()).await?;
    info!("Successfully updated EdDSA public key for JWT authentication");
    Ok(())
}

async fn update_root(login: String, password: Option<String>) -> Result<(), AppError> {
    DB.query(include_str!("../queries/update_root.surql"))
        .bind(("login", login))
        .bind(("password", password))
        .await?
        .check()?;
    info!("Successfully updated root user");
    Ok(())
}

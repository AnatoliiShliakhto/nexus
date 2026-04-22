use crate::domain::models::{AppliedMigration, Migration, MigrationReport};
use crate::error::{AppError, AppErrorExt};
use crate::generated::migrations_manifest::builtin_migrations;
use fxhash::FxHashMap;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

pub(crate) trait MigrationRunner {
    async fn migrate(&self) -> Result<MigrationReport, AppError>;
    async fn apply_migration(&self, migration: &Migration) -> Result<(), AppError>;
    async fn is_system_ready(&self) -> Result<bool, AppError>;
    async fn get_migrations_map(&self) -> Result<FxHashMap<String, AppliedMigration>, AppError>;
}

impl MigrationRunner for Surreal<Any> {
    async fn migrate(&self) -> Result<MigrationReport, AppError> {
        let mut report = MigrationReport::default();
        let migrations = builtin_migrations();
        let applied_migrations = self.get_migrations_map().await?;

        for migration in migrations {
            if let Some(applied) =
                applied_migrations.get(&format!("{}:{}", migration.component, migration.version))
            {
                ensure_checksum_match(&migration, &applied.checksum)?;
                report.skipped.push(migration.to_applied());
                continue;
            }

            self.apply_migration(&migration).await?;
            report.applied.push(migration.to_applied());
        }

        Ok(report)
    }

    async fn apply_migration(&self, migration: &Migration) -> Result<(), AppError> {
        let query = if migration.bootstrap {
            format!(
                "BEGIN TRANSACTION;
                {}
                fn::ensure_component($component, $name, $desc, $restricted);
                RETURN fn::confirm_migration($component, $version, $checksum);
                COMMIT TRANSACTION;",
                migration.sql,
            )
        } else {
            format!(
                "BEGIN TRANSACTION;
                fn::ensure_component($component, $name, $description, $restricted);
                {}
                RETURN fn::confirm_migration($component, $version, $checksum);
                COMMIT TRANSACTION;",
                migration.sql,
            )
        };

        let _ = self
            .query(&query)
            .bind(("component", migration.component))
            .bind(("name", migration.component_name))
            .bind(("description", migration.component_description))
            .bind(("version", migration.version))
            .bind(("checksum", migration.checksum))
            .bind(("restricted", migration.restricted))
            .await
            .with_help_fn(|| {
                format!(
                    "SQL execution failed at `{}`->`{}`",
                    &migration.component, &migration.version
                )
            })?
            .check()
            .with_help_fn(|| {
                format!("migration failed at `{}`->`{}`", &migration.component, &migration.version)
            })?;

        Ok(())
    }

    async fn is_system_ready(&self) -> Result<bool, AppError> {
        let mut response = self
            .query("!(SELECT VALUE fields FROM ONLY INFO FOR TABLE component).is_empty()")
            .await
            .with_help("Checking if system is ready")?
            .check()
            .with_help("Failed to check system status")?;

        let is_ready = response.take::<Option<bool>>(0)?.unwrap_or_default();
        Ok(is_ready)
    }

    async fn get_migrations_map(&self) -> Result<FxHashMap<String, AppliedMigration>, AppError> {
        let is_ready = self.is_system_ready().await?;

        if !is_ready {
            return Ok(FxHashMap::default());
        }

        let entries = self
            .query("SELECT id[0].id() as component, version, checksum FROM migration")
            .await
            .with_help("Transport error while loading migrations")?
            .check()
            .with_help("Database failed to execute migration query")?
            .take::<Vec<AppliedMigration>>(0)
            .with_help("Failed to map database rows to AppliedMigration struct")?;

        Ok(entries
            .into_iter()
            .map(|entry| (format!("{}:{}", entry.component, entry.version), entry))
            .collect())
    }
}

pub(crate) trait AccessRunner {
    async fn update_pub_key(&self, public_key: impl Into<String>) -> Result<(), AppError>;
}

impl AccessRunner for Surreal<Any> {
    async fn update_pub_key(&self, public_key: impl Into<String>) -> Result<(), AppError> {
        self.query(include_str!("../../ops/queries/update_key.surql"))
            .bind(("public_key", public_key.into()))
            .await
            .with_help("Failed to update public key")?
            .check()
            .with_help("Failed to update public key")?;

        Ok(())
    }
}

fn ensure_checksum_match(migration: &Migration, existing: &str) -> Result<(), AppError> {
    if existing != migration.checksum {
        Err(AppError::migration().with_help_fn(|| {
            format!("Checksum mismatch for `{}`->`{}`", migration.component, migration.version)
        }))?;
    }
    Ok(())
}

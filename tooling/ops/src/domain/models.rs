use surrealdb::types::SurrealValue;

pub(crate) struct Migration {
    pub component: &'static str,
    pub component_name: &'static str,
    pub component_description: Option<&'static str>,
    pub version: &'static str,
    pub sql: &'static str,
    pub checksum: &'static str,
    pub bootstrap: bool,
    pub restricted: bool,
}

#[derive(Debug, Default)]
pub(crate) struct MigrationReport {
    pub applied: Vec<AppliedMigration>,
    pub skipped: Vec<AppliedMigration>,
}

#[derive(Debug, SurrealValue)]
pub(crate) struct AppliedMigration {
    pub component: String,
    pub version: String,
    pub checksum: String,
}

impl Migration {
    pub(crate) fn to_applied(&self) -> AppliedMigration {
        AppliedMigration {
            component: self.component.to_owned(),
            version: self.version.to_owned(),
            checksum: self.checksum.to_owned(),
        }
    }
}

use crate::config::VersionFormatting;
use std::collections::BTreeMap;
use std::fs::read_dir;

use crate::migration::extractor::extract_migration;
use crate::migration::FileError::*;
use crate::migration::MigrationParsingError::*;
use crate::migration::{
    FileError, Migration, MigrationParsingError, MigrationResult, MigrationVersion,
};

struct MigrationStack {
    migrations: BTreeMap<MigrationVersion, Migration>,
    errors: Vec<MigrationParsingError>,
}

impl MigrationStack {
    fn new() -> Self {
        MigrationStack {
            migrations: BTreeMap::new(),
            errors: Vec::new(),
        }
    }

    fn push_migration(&mut self, migration: Migration) {
        let version = migration.version.clone();
        let migration_name = migration.filename.clone();

        self.migrations
            .insert(version, migration)
            .map(|_| self.push_error(DuplicatedMigrationError { migration_name }));
    }

    fn push_error(&mut self, error: MigrationParsingError) {
        self.errors.push(error);
    }

    fn into_result(self) -> MigrationResult {
        MigrationResult {
            errors: self.errors,
            migrations: self.migrations,
        }
    }
}

pub fn validate_migrations(
    directory_path: &str,
    version_formatting: &VersionFormatting,
) -> Result<MigrationResult, FileError> {
    let entries = read_dir(directory_path).map_err(|_err| DirectoryNotLoadedError)?;
    let mut migration_stack = MigrationStack::new();

    for entry in entries {
        let entry = entry.map_err(|_err| FileNotLoadedError)?;

        match extract_migration(&entry, version_formatting) {
            Ok(migration) => migration_stack.push_migration(migration),
            Err(err) => migration_stack.push_error(err),
        }
    }

    Ok(migration_stack.into_result())
}

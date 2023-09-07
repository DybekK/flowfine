use crate::migration::version::MigrationVersionKey;
use crate::migration::MigrationParsingError::*;
use std::collections::BTreeMap;
use thiserror::Error;

pub mod lexer;
pub mod parser;
pub mod version;

#[derive(Debug, Error)]
pub enum FileError {
    #[error("Directory not loaded")]
    DirectoryNotLoadedError,

    #[error("File not loaded")]
    FileNotLoadedError,
}

#[derive(Debug, PartialEq, Eq, Hash, Error)]
pub enum MigrationParsingError {
    #[error("Duplicated migration.rs version for file: {0}")]
    DuplicatedMigrationError(String),

    #[error("Invalid migration.rs format for file: {0}")]
    InvalidMigrationFormatError(String),

    #[error("Invalid version format for file {0}")]
    InvalidVersionFormatError(String),

    #[error("Missing migration.rs content for file {0}")]
    MissingMigrationContentError(String),

    #[error("Missing semicolons in migration.rs content for file {0}")]
    NoSemicolonsFoundError(String),
}

#[derive(Clone, Debug)]
pub struct Migration {
    pub filename: String,
    pub version: String,
    pub version_key: MigrationVersionKey,
    pub name: String,
    pub content: String,
    pub queries: Vec<String>,
}

pub struct MigrationResult {
    errors: Vec<MigrationParsingError>,
    migrations: Vec<Migration>,
}

impl MigrationResult {
    pub fn into_result(self) -> Result<Vec<Migration>, Vec<MigrationParsingError>> {
        if self.errors.is_empty() {
            Ok(self.migrations)
        } else {
            Err(self.errors)
        }
    }
}

pub struct MigrationStack {
    migrations: BTreeMap<MigrationVersionKey, Migration>,
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
        let filename = migration.filename.clone();
        let version_key = migration.version_key.clone();

        self.migrations
            .insert(version_key, migration)
            .map(|_| self.push_error(DuplicatedMigrationError(filename)));
    }

    fn push_error(&mut self, error: MigrationParsingError) {
        self.errors.push(error);
    }

    fn into_result(self) -> MigrationResult {
        MigrationResult {
            errors: self.errors,
            migrations: self.migrations.into_values().collect(),
        }
    }
}

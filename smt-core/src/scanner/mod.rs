use std::collections::BTreeMap;

use crate::scanner::version::MigrationVersionKey;
use crate::scanner::MigrationParsingError::DuplicatedMigrationError;
use thiserror::Error;

pub mod parser;
mod version;

#[derive(Debug, Error)]
pub enum FileError {
    #[error("Directory not loaded")]
    DirectoryNotLoadedError,

    #[error("File not loaded")]
    FileNotLoadedError,
}

#[derive(Debug, Error)]
pub enum MigrationParsingError {
    #[error("Duplicated migration version for file: {0}")]
    DuplicatedMigrationError(String),

    #[error("Invalid migration format for file: {0}")]
    InvalidMigrationFormatError(String),

    #[error("Invalid version format for file {0}")]
    InvalidVersionFormatError(String),

    #[error("Missing migration content for file {0}")]
    MissingMigrationContentError(String),
}

#[derive(Clone, Debug)]
pub struct Migration {
    pub filename: String,
    pub version: String,
    pub name: String,
    pub content: String,
}

pub struct MigrationResult {
    errors: Vec<MigrationParsingError>,
    migrations: Vec<Migration>,
}

impl MigrationResult {
    pub fn print_report(self) {
        if !self.errors.is_empty() {
            println!("Migration Errors:");
            for error in &self.errors {
                println!("{:?}", error.to_string());
            }
        } else {
            println!("Migration Report:");
            for migration in &self.migrations {
                println!("Migration: {:?}", migration);
            }
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

    fn push_migration(&mut self, version_key: MigrationVersionKey, migration: Migration) {
        let filename = migration.filename.clone();

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

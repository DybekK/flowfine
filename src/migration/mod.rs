use crate::migration::version::MigrationVersion;
use std::collections::BTreeMap;
use thiserror::Error;

mod extractor;
mod version;

pub mod validator;

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
    pub version: MigrationVersion,
    pub name: String,
    pub content: String,
}

pub struct MigrationResult {
    errors: Vec<MigrationParsingError>,
    migrations: BTreeMap<MigrationVersion, Migration>,
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
            for (_, migration) in &self.migrations {
                println!("Migration: {:?}", migration);
            }
        }
    }
}

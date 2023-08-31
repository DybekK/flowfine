use std::collections::BTreeMap;
use thiserror::Error;

mod extractor;
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
    #[error("Duplicated migration version in file: {migration_name}")]
    DuplicatedMigrationError { migration_name: String },

    #[error("Invalid migration format")]
    InvalidMigrationFormatError,

    #[error("Missing migration content")]
    MissingMigrationContentError,
}

pub struct Migration {
    pub filename: String,
    pub version: String,
    pub name: String,
    pub content: String,
}

pub struct MigrationResult {
    errors: Vec<MigrationParsingError>,
    migrations: BTreeMap<String, Migration>,
}

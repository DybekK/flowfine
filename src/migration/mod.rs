use std::cmp::Ordering;
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

#[derive(Clone, PartialOrd, PartialEq, Eq)]
pub struct NumericVersion(String);

impl Ord for NumericVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Clone, PartialOrd, PartialEq, Eq, Ord)]
pub enum MigrationVersion {
    Numeric(NumericVersion),
}

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

use crate::config::VersionFormatting;
use crate::migration::MigrationVersion::Numeric;
use regex::Regex;
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

    #[error("Invalid version format")]
    InvalidVersionFormatError,

    #[error("Missing migration content")]
    MissingMigrationContentError,
}

#[derive(Clone, PartialOrd, PartialEq, Eq, Ord)]
pub enum MigrationVersion {
    Numeric(NumericVersion),
}

impl MigrationVersion {
    pub fn new(
        version_formatting: &VersionFormatting,
        version: &str,
    ) -> Result<Self, MigrationParsingError> {
        match version_formatting {
            VersionFormatting::Numeric => NumericVersion::new(version),
            VersionFormatting::Datetime => unimplemented!(),
        }
    }
}

#[derive(Clone, PartialOrd, PartialEq, Eq)]
pub struct NumericVersion(String);

impl NumericVersion {
    fn new(version: &str) -> Result<MigrationVersion, MigrationParsingError> {
        let version_regex = Regex::new(r"^\d+\.\d+$").unwrap();
        if version_regex.is_match(version) {
            Ok(Numeric(NumericVersion(version.to_string())))
        } else {
            Err(MigrationParsingError::InvalidVersionFormatError)
        }
    }
}

impl Ord for NumericVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
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

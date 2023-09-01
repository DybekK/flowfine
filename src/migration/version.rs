use crate::config::VersionFormatting;
use crate::migration::MigrationParsingError;
use crate::migration::MigrationParsingError::*;
use chrono::NaiveDateTime;
use regex::Regex;

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub enum MigrationVersion {
    Numeric(NumericVersion),
    Datetime(DatetimeVersion),
}

impl MigrationVersion {
    pub fn new(
        version_formatting: &VersionFormatting,
        filename: &str,
        version: &str,
    ) -> Result<Self, MigrationParsingError> {
        match version_formatting {
            VersionFormatting::Numeric => NumericVersion::new(filename, version),
            VersionFormatting::Datetime => DatetimeVersion::new(filename, version),
        }
    }
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct NumericVersion(String);

impl NumericVersion {
    fn new(filename: &str, version: &str) -> Result<MigrationVersion, MigrationParsingError> {
        let version_regex = Regex::new(r"^(\d{1,10}(\.\d+)?|\d+\.\d+)$").unwrap();

        if version_regex.is_match(version) {
            Ok(MigrationVersion::Numeric(NumericVersion(
                version.to_string(),
            )))
        } else {
            Err(InvalidVersionFormatError(filename.to_string()))
        }
    }
}
#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct DatetimeVersion(NaiveDateTime);

impl DatetimeVersion {
    fn new(filename: &str, version: &str) -> Result<MigrationVersion, MigrationParsingError> {
        match NaiveDateTime::parse_from_str(version, "%Y%m%d%H%M%S") {
            Ok(datetime) => Ok(MigrationVersion::Datetime(DatetimeVersion(datetime))),
            Err(_) => Err(InvalidVersionFormatError(filename.to_string())),
        }
    }
}

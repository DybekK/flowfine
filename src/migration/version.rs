use crate::config::VersionFormatting;
use crate::migration::MigrationParsingError;
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

trait MigrationVersionValidator {
    fn new(filename: &str, version: &str) -> Result<MigrationVersion, MigrationParsingError>;
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct NumericVersion(String);

impl MigrationVersionValidator for NumericVersion {
    fn new(filename: &str, version: &str) -> Result<MigrationVersion, MigrationParsingError> {
        let version_regex = Regex::new(r"^(\d{1,10}(\.\d+)?|\d+\.\d+)$").unwrap();

        if version_regex.is_match(version) {
            Ok(MigrationVersion::Numeric(NumericVersion(
                version.to_string(),
            )))
        } else {
            Err(MigrationParsingError::InvalidVersionFormatError {
                name: filename.to_string(),
            })
        }
    }
}
#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct DatetimeVersion(NaiveDateTime);

impl MigrationVersionValidator for DatetimeVersion {
    fn new(filename: &str, version: &str) -> Result<MigrationVersion, MigrationParsingError> {
        match NaiveDateTime::parse_from_str(version, "%Y%m%d%H%M%S") {
            Ok(datetime) => Ok(MigrationVersion::Datetime(DatetimeVersion(datetime))),
            Err(_) => Err(MigrationParsingError::InvalidVersionFormatError {
                name: filename.to_string(),
            }),
        }
    }
}

use crate::config::VersionFormatting;

use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref NUMERIC_VERSION_REGEX: Regex =
        Regex::new(r"^(\d{1,10}(\.\d+)*|\d+(\.\d+)+)$").unwrap();
    static ref DATETIME_VERSION_FORMAT: String = "%Y%m%d%H%M%S".to_string();
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub enum MigrationVersionKey {
    Numeric(String),
    Datetime(NaiveDateTime),
}

impl MigrationVersionKey {
    pub fn new(version_formatting: &VersionFormatting, version: &str) -> Option<Self> {
        match version_formatting {
            VersionFormatting::Numeric => parse_numeric_version(version),
            VersionFormatting::Datetime => parse_datetime_version(version),
        }
    }
}

fn parse_numeric_version(version: &str) -> Option<MigrationVersionKey> {
    if NUMERIC_VERSION_REGEX.is_match(version) {
        Some(MigrationVersionKey::Numeric(version.to_string()))
    } else {
        None
    }
}

fn parse_datetime_version(version: &str) -> Option<MigrationVersionKey> {
    match NaiveDateTime::parse_from_str(version, &DATETIME_VERSION_FORMAT) {
        Ok(datetime) => Some(MigrationVersionKey::Datetime(datetime)),
        Err(_) => None,
    }
}

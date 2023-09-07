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
            VersionFormatting::Numeric => Self::parse_numeric_version(version),
            VersionFormatting::Datetime => Self::parse_datetime_version(version),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(version, expected_result,
    case("1", MigrationVersionKey::Numeric("1".to_string())),
    case("1.2", MigrationVersionKey::Numeric("1.2".to_string())),
    case("1.2.3",MigrationVersionKey::Numeric("1.2.3".to_string())),
    )]
    fn test_valid_numeric_version(version: &str, expected_result: MigrationVersionKey) {
        // when
        let migrated_version = MigrationVersionKey::new(&VersionFormatting::Numeric, version);

        // then
        assert!(migrated_version.is_some());
        assert_eq!(migrated_version.unwrap(), expected_result);
    }

    #[rstest(version, case(".1"), case("1."), case("1..2"), case("1.2.a"))]
    fn test_invalid_numeric_version(version: &str) {
        // when
        let migrated_version = MigrationVersionKey::new(&VersionFormatting::Numeric, version);

        // then
        assert!(migrated_version.is_none());
    }

    #[rstest(version, expected_result,
    case("20210903120000", MigrationVersionKey::Datetime(NaiveDateTime::parse_from_str("20210903120000", &DATETIME_VERSION_FORMAT).unwrap())),
    case("20211231235959", MigrationVersionKey::Datetime(NaiveDateTime::parse_from_str("20211231235959", &DATETIME_VERSION_FORMAT).unwrap())),
    )]
    fn test_valid_datetime_version(version: &str, expected_result: MigrationVersionKey) {
        // when
        let migrated_version = MigrationVersionKey::new(&VersionFormatting::Datetime, version);

        // then
        assert!(migrated_version.is_some());
        assert_eq!(migrated_version.unwrap(), expected_result);
    }

    #[rstest(version,
    case("20211301120000"),  // Invalid month (13)
    case("20210132000000"),  // Invalid day (32)
    case("202101011200000"), // Invalid time (too many digits)
    case("2021010112"),      // Incomplete time (no seconds)
    case("20210101120000x"), // Non-numeric character
    )]
    fn test_invalid_datetime_version(version: &str) {
        // when
        let migrated_version = MigrationVersionKey::new(&VersionFormatting::Datetime, version);

        // then
        assert!(migrated_version.is_none());
    }
}

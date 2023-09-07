use crate::config::VersionFormatting;
use std::fs::{read_dir, read_to_string, DirEntry};

use crate::scanner::query::delimit_queries;
use crate::scanner::FileError::*;
use crate::scanner::MigrationParsingError::*;
use crate::scanner::*;

pub fn parse_migrations(
    directory_path: &str,
    version_formatting: &VersionFormatting,
) -> Result<MigrationResult, FileError> {
    let entries = read_dir(directory_path).map_err(|_err| DirectoryNotLoadedError)?;
    let mut migration_stack = MigrationStack::new();

    for entry in entries {
        let entry = entry.map_err(|_err| FileNotLoadedError)?;

        match parse_migration(&entry, version_formatting) {
            Ok(migration) => migration_stack.push_migration(migration),
            Err(err) => migration_stack.push_error(err),
        }
    }

    Ok(migration_stack.into_result())
}

fn parse_migration(
    path: &DirEntry,
    version_formatting: &VersionFormatting,
) -> Result<Migration, MigrationParsingError> {
    let filename = parse_migration_filename(path);
    let (version, version_key) = parse_migration_version(&filename, version_formatting)?;
    let name = parse_migration_name(&filename)?;
    let content = parse_migration_content(path)?;
    let queries = delimit_queries(&filename, &content)?;

    let migration = Migration {
        filename,
        version,
        version_key,
        name,
        content,
        queries,
    };

    Ok(migration)
}

fn parse_migration_filename(path: &DirEntry) -> String {
    match path.file_name().into_string() {
        Ok(filename) => filename,
        Err(filename) => filename.to_string_lossy().to_string(),
    }
}

fn parse_migration_version(
    filename: &str,
    version_formatting: &VersionFormatting,
) -> Result<(String, MigrationVersionKey), MigrationParsingError> {
    let start = 1;
    let end = filename
        .find("__")
        .ok_or(InvalidMigrationFormatError(filename.to_string()))?;
    let version = &filename[start..end];

    MigrationVersionKey::new(version_formatting, version)
        .map(|version_key| (version.to_string(), version_key))
        .ok_or(InvalidVersionFormatError(filename.to_string()))
}

fn parse_migration_name(filename: &str) -> Result<String, MigrationParsingError> {
    let extension = ".cql";

    if !filename.ends_with(extension) {
        return Err(InvalidMigrationFormatError(filename.to_string()));
    }

    let migration_name = filename
        .find("__")
        .and_then(|start| filename.rfind(extension).map(|end| (start, end)))
        .map(|(start, end)| &filename[start + 2..end])
        .ok_or(InvalidMigrationFormatError(filename.to_string()))?;

    Ok(migration_name.replace("_", " "))
}

fn parse_migration_content(path: &DirEntry) -> Result<String, MigrationParsingError> {
    match read_to_string(path.path()) {
        Ok(content) if !content.trim().is_empty() => Ok(content),
        Ok(_) => Err(MissingMigrationContentError(parse_migration_filename(path))),
        Err(_) => Err(MissingMigrationContentError(parse_migration_filename(path))),
    }
}

// todo: write tests for checking migration's queries
#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use rstest::rstest;
    use std::collections::HashSet;

    #[rstest(version_formatting, path, expected_result,
        case(VersionFormatting::Numeric, "./tests/data/scanner/numeric_migrations", vec![
            "V1__migration.cql",
            "V1.1__migration.cql",
            "V2.0__migration.cql", 
        ]),
        case(VersionFormatting::Datetime, "./tests/data/scanner/datetime_migrations", vec![
            "V20230903141500__migration.cql",
            "V20230903141501__migration.cql",
            "V20230903141502__migration.cql",
        ])
    )]
    fn test_valid_migrations(
        version_formatting: VersionFormatting,
        path: &str,
        expected_result: Vec<&str>,
    ) {
        // when
        let result = parse_migrations(path, &version_formatting);

        // then
        assert!(result.is_ok());
        assert_migrations(expected_result, result.unwrap().migrations);
    }

    #[test]
    fn test_invalid_migrations() {
        // given
        let version_formatting = VersionFormatting::Numeric;
        let path = "./tests/data/scanner/invalid_migrations";

        // when
        let result = parse_migrations(path, &version_formatting);

        // then
        assert!(result.is_ok());
        let expected = vec![
            InvalidVersionFormatError("V__invalid_migration_version.cql".to_string()),
            InvalidMigrationFormatError("V1_invalid_migration_underscore.cql".to_string()),
            MissingMigrationContentError("V1__invalid_migration_missing_content.cql".to_string()),
            InvalidMigrationFormatError("V1__invalid_migration_missing_extension.".to_string()),
            NoSemicolonsFoundError("V1__invalid_migration_missing_semicolon.cql".to_string()),
        ];

        assert_errors_any_order(expected, result.unwrap().errors);
    }

    fn assert_migrations(expected: Vec<&str>, actual: Vec<Migration>) {
        let actual_filenames = actual
            .into_iter()
            .into_iter()
            .map(|migration| migration.filename)
            .collect_vec();

        assert_eq!(expected, actual_filenames);
    }

    fn assert_errors_any_order(
        expected: Vec<MigrationParsingError>,
        actual: Vec<MigrationParsingError>,
    ) {
        assert_eq!(
            expected.into_iter().collect::<HashSet<_>>(),
            actual.into_iter().collect::<HashSet<_>>(),
        );
    }
}

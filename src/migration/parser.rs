use crate::config::VersionFormatting;
use std::fs::{read_dir, read_to_string, DirEntry};

use crate::migration::MigrationParsingError::*;

use crate::migration::FileError::*;
use crate::migration::*;

pub fn parse_migrations(
    directory_path: &str,
    version_formatting: &VersionFormatting,
) -> Result<MigrationResult, FileError> {
    let entries = read_dir(directory_path).map_err(|_err| DirectoryNotLoadedError)?;
    let mut migration_stack = MigrationStack::new();

    for entry in entries {
        let entry = entry.map_err(|_err| FileNotLoadedError)?;

        match parse_migration(&entry, version_formatting) {
            Ok((version_key, migration)) => migration_stack.push_migration(version_key, migration),
            Err(err) => migration_stack.push_error(err),
        }
    }

    Ok(migration_stack.into_result())
}

fn parse_migration(
    path: &DirEntry,
    version_formatting: &VersionFormatting,
) -> Result<(MigrationVersionKey, Migration), MigrationParsingError> {
    let filename = parse_migration_filename(path);
    let (version, version_key) = parse_migration_version(&filename, version_formatting)?;
    let name = parse_migration_name(&filename)?;
    let content = parse_migration_content(path)?;
    let migration = Migration {
        filename,
        version,
        name,
        content,
    };

    Ok((version_key, migration))
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
    let migration_name = filename
        .find("__")
        .and_then(|start| filename.rfind(".cql").map(|end| (start, end)))
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

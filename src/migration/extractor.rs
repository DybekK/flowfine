use crate::config::VersionFormatting;
use std::fs::{read_to_string, DirEntry};

use crate::migration::MigrationParsingError::*;

use crate::migration::{Migration, MigrationParsingError, MigrationVersion};

pub fn extract_migration(
    path: &DirEntry,
    version_formatting: &VersionFormatting,
) -> Result<Migration, MigrationParsingError> {
    let filename = get_migration_filename(path);
    let version = get_migration_version(&filename, version_formatting)?;
    let name = get_migration_name(&filename)?;
    let content = get_migration_content(path)?;
    let migration = Migration {
        filename,
        version,
        name,
        content,
    };

    Ok(migration)
}

fn get_migration_filename(path: &DirEntry) -> String {
    match path.file_name().into_string() {
        Ok(filename) => filename,
        Err(filename) => filename.to_string_lossy().to_string(),
    }
}

fn get_migration_version(
    filename: &str,
    version_formatting: &VersionFormatting,
) -> Result<MigrationVersion, MigrationParsingError> {
    let start = 1;
    let end = filename.find("__").ok_or(InvalidMigrationFormatError {
        name: filename.to_string(),
    })?;
    let version = &filename[start..end];

    MigrationVersion::new(version_formatting, filename, version)
}

fn get_migration_name(filename: &str) -> Result<String, MigrationParsingError> {
    let migration_name = filename
        .find("__")
        .and_then(|start| filename.rfind(".cql").map(|end| (start, end)))
        .map(|(start, end)| &filename[start + 2..end])
        .ok_or(InvalidMigrationFormatError {
            name: filename.to_string(),
        })?;

    Ok(migration_name.replace("_", " "))
}

fn get_migration_content(path: &DirEntry) -> Result<String, MigrationParsingError> {
    match read_to_string(path.path()) {
        Ok(content) if !content.trim().is_empty() => Ok(content),
        Ok(_) => Err(MissingMigrationContentError {
            name: get_migration_filename(path),
        }),
        Err(_) => Err(MissingMigrationContentError {
            name: get_migration_filename(path),
        }),
    }
}

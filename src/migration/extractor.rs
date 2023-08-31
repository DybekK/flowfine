use std::fs::{read_to_string, DirEntry};

use crate::migration::MigrationParsingError::*;
use crate::migration::MigrationVersion::Numeric;
use crate::migration::{Migration, MigrationParsingError, MigrationVersion, NumericVersion};

pub fn extract_migration(path: &DirEntry) -> Result<Migration, MigrationParsingError> {
    let filename = get_migration_filename(path);
    let version = get_migration_version(&filename)?;
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

fn get_migration_version(filename: &str) -> Result<MigrationVersion, MigrationParsingError> {
    let start = 1;
    let end = filename.find("__").ok_or(InvalidMigrationFormatError)?;
    let version = &filename[start..end];

    Ok(Numeric(NumericVersion(version.to_string())))
}

fn get_migration_name(filename: &str) -> Result<String, MigrationParsingError> {
    let migration_name = filename
        .find("__")
        .and_then(|start| filename.rfind(".cql").map(|end| (start, end)))
        .map(|(start, end)| &filename[start + 2..end])
        .ok_or(InvalidMigrationFormatError)?;

    Ok(migration_name.replace("_", " "))
}

fn get_migration_content(path: &DirEntry) -> Result<String, MigrationParsingError> {
    match read_to_string(path.path()) {
        Ok(content) if !content.trim().is_empty() => Ok(content),
        Ok(_) => Err(MissingMigrationContentError),
        Err(_) => Err(MissingMigrationContentError),
    }
}

use crate::config::MigrationConfig;
use crate::scanner::parser::parse_migrations;

pub mod config;
mod scanner;

pub async fn migrate(config: MigrationConfig) {
    match parse_migrations(&config.directory, &config.version_formatting) {
        Ok(migration_result) => migration_result.print_report(),
        Err(err) => println!("Error: {:?}", err),
    }
}

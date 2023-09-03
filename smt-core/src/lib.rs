use crate::config::SmtConfig;
use crate::scanner::parser::parse_migrations;

pub mod config;
mod scanner;

pub async fn migrate(config: SmtConfig) {
    match parse_migrations(&config.directory, &config.version_formatting) {
        Ok(migration_result) => migration_result.print_report(),
        Err(err) => println!("Error: {:?}", err),
    }
}

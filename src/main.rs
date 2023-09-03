use crate::config::SmtConfig;
use crate::config::VersionFormatting::{Datetime, Numeric};
use crate::migration::parser::parse_migrations;

mod config;
mod migration;

#[tokio::main]
async fn main() {
    let config = SmtConfig {
        username: "".to_string(),
        password: "".to_string(),
        directory: "./migrations".to_string(),
        version_formatting: Numeric,
        keyspace: "smt".to_string(),
        host: "localhost".to_string(),
        port: 9042,
    };
    migrate(&config);
}

fn migrate(config: &SmtConfig) {
    match parse_migrations(&config.directory, &config.version_formatting) {
        Ok(migration_result) => migration_result.print_report(),
        Err(err) => println!("Error: {:?}", err),
    }
}

use crate::config::SmtConfig;
use crate::migration::validator::validate_migrations;

mod config;
mod migration;

#[tokio::main]
async fn main() {
    let config = SmtConfig {
        username: "".to_string(),
        password: "".to_string(),
        directory: "./migrations".to_string(),
        keyspace: "smt".to_string(),
        host: "localhost".to_string(),
        port: 9042,
    };
    migrate(&config);
}

fn migrate(config: &SmtConfig) {
    validate_migrations(&config.directory);
}
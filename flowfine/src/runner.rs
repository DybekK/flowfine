use crate::runner::MigrationExecutionError::*;
use crate::scanner::Migration;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use lazy_static::lazy_static;
use scylla::frame::value::Timestamp;
use scylla::transport::errors::QueryError;
use scylla::{QueryResult, Session};
use sha2::{Digest, Sha256};
use thiserror::Error;

lazy_static! {
    static ref HISTORY_TABLE_NAME: String = "flowfine_history".to_string();
}

#[derive(Error, Debug)]
pub enum MigrationExecutionError {
    #[error("Migration table was not created: {0}")]
    CreateHistoryTableError(QueryError),

    #[error("Migration {0} failed: {1}")]
    RunMigrationError(String, QueryError),

    #[error("Migration history could not be applied: {0}")]
    ApplyHistoryError(QueryError),
}

struct AppliedMigration {
    version: String,
    name: String,
    checksum: String,
    applied_at: Duration,
    success: bool,
}

#[async_trait]
pub trait MigrationRunner {
    async fn run(&self, migrations: Vec<Migration>) -> Result<(), MigrationExecutionError>;
}

pub struct ScyllaMigrationRunner<'a> {
    session: &'a Session,
    keyspace: String,
}

impl<'a> ScyllaMigrationRunner<'a> {
    pub fn new(session: &'a Session, keyspace: &str) -> Self {
        Self {
            session,
            keyspace: keyspace.to_string(),
        }
    }

    async fn apply_migration(&self, migration: &Migration) -> Result<(), MigrationExecutionError> {
        for query in &migration.queries {
            self.session
                .query(query.clone(), &[])
                .await
                .map_err(|err| RunMigrationError(migration.filename.clone(), err.clone()))?;
        }

        Ok(())
    }

    async fn apply_history(
        &self,
        success: bool,
        migration: &Migration,
    ) -> Result<QueryResult, MigrationExecutionError> {
        let query = format!(
            "INSERT INTO {keyspace}.{history_table} (version, name, checksum, applied_at, success) VALUES (?, ?, ?, ?, ?);",
            keyspace = self.keyspace,
            history_table = *HISTORY_TABLE_NAME
        );

        let applied_migration = AppliedMigration {
            version: migration.version.clone(),
            name: migration.name.clone(),
            checksum: self.calculate_checksum(&migration),
            applied_at: Duration::seconds(Utc::now().timestamp()), //todo: move the code to date utils
            success,
        };

        self.session
            .query(
                query,
                (
                    &applied_migration.version,
                    &applied_migration.name,
                    &applied_migration.checksum,
                    Timestamp(applied_migration.applied_at),
                    &applied_migration.success,
                ),
            )
            .await
            .map_err(|err| ApplyHistoryError(err.clone()))
    }

    fn calculate_checksum(&self, migration: &Migration) -> String {
        let checksum = Sha256::new()
            .chain_update(migration.version.as_bytes())
            .chain_update(migration.name.as_bytes())
            .chain_update(migration.content.as_bytes())
            .finalize();

        format!("{:x}", checksum)
    }

    async fn create_history_table(&self) -> Result<QueryResult, MigrationExecutionError> {
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {keyspace}.{history_table} (
                version    TEXT PRIMARY KEY,
                name       TEXT, 
                checksum   TEXT,
                applied_at TIMESTAMP,
                success    BOOLEAN
            );",
            keyspace = self.keyspace,
            history_table = *HISTORY_TABLE_NAME
        );

        self.session
            .query(query, &[])
            .await
            .map_err(|err| CreateHistoryTableError(err))
    }
}

#[async_trait]
impl<'a> MigrationRunner for ScyllaMigrationRunner<'a> {
    async fn run(&self, migrations: Vec<Migration>) -> Result<(), MigrationExecutionError> {
        self.create_history_table().await?;

        for migration in migrations {
            match self.apply_migration(&migration).await {
                Ok(_) => self.apply_history(true, &migration).await?,
                Err(err) => {
                    self.apply_history(false, &migration).await?;
                    return Err(err);
                }
            };
        }

        Ok(())
    }
}

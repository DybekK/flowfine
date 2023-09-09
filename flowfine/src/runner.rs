use crate::migration::Migration;
use crate::runner::MigrationExecutionError::*;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use lazy_static::lazy_static;
use nanoid::nanoid;
use scylla::frame::value::Timestamp;
use scylla::transport::errors::QueryError;
use scylla::{FromRow, QueryResult, Session};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use thiserror::Error;

lazy_static! {
    static ref NANOID_LENGTH: usize = 15;
    static ref HISTORY_TABLE_NAME: String = "flowfine_history".to_string();
}

#[derive(Error, Debug)]
pub enum MigrationExecutionError {
    #[error("Migration table was not created: {0}")]
    CreateHistoryTableError(QueryError),

    #[error("Migration {0} failed: {1}")]
    RunMigrationError(String, QueryError),

    #[error("")]
    MigrationError(QueryError),

    #[error("Migration history could not be applied: {0}")]
    ApplyHistoryError(QueryError),
}

#[derive(FromRow)]
pub struct AppliedMigration {
    pub id: String,
    pub version: String,
    pub name: String,
    pub filename: String,
    pub checksum: String,
    pub applied_at: Duration,
    pub success: bool,
}

#[async_trait]
pub trait MigrationRunner {
    async fn run(
        &self,
        migrations: Vec<Migration>,
    ) -> Result<Vec<AppliedMigration>, MigrationExecutionError>;
}

pub struct ScyllaMigrationRunner {
    session: Arc<Session>,
    keyspace: String,
}

impl<'a> ScyllaMigrationRunner {
    pub fn new(session: Arc<Session>, keyspace: &str) -> Self {
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
    ) -> Result<AppliedMigration, MigrationExecutionError> {
        let query = format!(
            "INSERT INTO {keyspace}.{history_table} (id, version, name, filename, checksum, applied_at, success) VALUES (?, ?, ?, ?, ?, ?, ?);",
            keyspace = self.keyspace,
            history_table = *HISTORY_TABLE_NAME
        );

        let nanoid_len = *NANOID_LENGTH;
        let applied_migration = AppliedMigration {
            id: nanoid!(nanoid_len).to_string(),
            version: migration.version.clone(),
            name: migration.name.clone(),
            filename: migration.filename.clone(),
            checksum: self.create_checksum(&migration),
            applied_at: Duration::nanoseconds(Utc::now().timestamp_nanos()), //todo: move the code to date utils
            success,
        };

        self.session
            .query(
                query,
                (
                    &applied_migration.id,
                    &applied_migration.version,
                    &applied_migration.name,
                    &applied_migration.filename,
                    &applied_migration.checksum,
                    Timestamp(applied_migration.applied_at),
                    &applied_migration.success,
                ),
            )
            .await
            .map(|_| applied_migration)
            .map_err(|err| ApplyHistoryError(err.clone()))
    }

    async fn find_latest_applied_migration(
        &self,
    ) -> Result<Option<AppliedMigration>, MigrationExecutionError> {
        let query = format!(
            "SELECT id, version, name, filename, checksum, applied_at, success
                FROM {keyspace}.{history_table} 
                WHERE success = true LIMIT 1 ALLOW FILTERING;
             ",
            keyspace = self.keyspace,
            history_table = *HISTORY_TABLE_NAME
        );

        self.session
            .query(query, &[])
            .await
            .map(|query_result| {
                query_result
                    .maybe_first_row_typed::<AppliedMigration>()
                    .unwrap()
            })
            .map_err(|err| MigrationError(err.clone()))
    }

    fn create_checksum(&self, migration: &Migration) -> String {
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
                id         TEXT,
                version    TEXT,
                name       TEXT, 
                filename   TEXT,
                checksum   TEXT,
                success    BOOLEAN,
                applied_at TIMESTAMP,
                PRIMARY KEY ((id, version, success), applied_at)
            ) WITH CLUSTERING ORDER BY (applied_at ASC);
            ",
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
impl<'a> MigrationRunner for ScyllaMigrationRunner {
    async fn run(
        &self,
        migrations: Vec<Migration>,
    ) -> Result<Vec<AppliedMigration>, MigrationExecutionError> {
        let mut applied_migrations = Vec::new();

        self.create_history_table().await?;
        let latest_migration = self.find_latest_applied_migration().await?;

        for migration in migrations {
            //todo: add property in config to verify integrity of all migrations
            if let Some(latest_migration) = &latest_migration {
                if latest_migration.version >= migration.version {
                    continue;
                }
            }

            match self.apply_migration(&migration).await {
                Ok(_) => {
                    let applied_migration = self.apply_history(true, &migration).await?;
                    applied_migrations.push(applied_migration);
                    println!("Applied migration.rs {}", migration.filename)
                }
                Err(err) => {
                    self.apply_history(false, &migration).await?;
                    println!("Failed to apply migration.rs {}", migration.filename);
                    return Err(err);
                }
            };
        }

        Ok(applied_migrations)
    }
}

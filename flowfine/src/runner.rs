use crate::runner::MigrationExecutionError::*;
use crate::scanner::Migration;
use async_trait::async_trait;
use scylla::{QueryResult, Session};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Hash, Error)]
pub enum MigrationExecutionError {
    #[error("Keyspace was not created")]
    CreateKeyspaceError,

    #[error("Migration table was not created")]
    CreateTableError,

    #[error("Migration failed")]
    MigrationExecutionError,
}

#[async_trait]
pub trait MigrationRunner {
    async fn run(&self, migrations: Vec<Migration>) -> Result<(), MigrationExecutionError>;
}

pub struct ScyllaMigrationRunner {
    session: Session,
    keyspace: String,
}

impl ScyllaMigrationRunner {
    pub fn new(session: Session, keyspace: String) -> Self {
        Self { session, keyspace }
    }

    async fn run_migration(&self, migration: Migration) -> Result<(), MigrationExecutionError> {
        for migration in migration.queries {
            self.session
                .query(migration, &[])
                .await
                .map(|_| ())
                .map_err(|_| MigrationExecutionError)?
        }

        Ok(())
    }

    async fn create_keyspace(&self) -> Result<QueryResult, MigrationExecutionError> {
        let query = format!(
            // todo: use prepared statements
            "CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor' : 1}};",
            self.keyspace,
        );

        self.session
            .query(query, &[])
            .await
            .map_err(|_| CreateKeyspaceError)
    }

    async fn create_revision(&self) -> Result<QueryResult, MigrationExecutionError> {
        let query = format!(
            // todo: use prepared statements
            "CREATE TABLE IF NOT EXISTS {}.flowfine_revision (id int PRIMARY KEY, name text);",
            self.keyspace,
        );

        self.session
            .query(query, &[])
            .await
            .map_err(|_| CreateTableError)
    }
}

#[async_trait]
impl MigrationRunner for ScyllaMigrationRunner {
    async fn run(&self, migrations: Vec<Migration>) -> Result<(), MigrationExecutionError> {
        self.create_keyspace().await?;
        self.create_revision().await?;

        for migration in migrations {
            self.run_migration(migration).await?;
        }

        Ok(())
    }
}

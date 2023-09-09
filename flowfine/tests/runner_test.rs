#[cfg(test)]
mod tests {
    use flowfine::config::VersionFormatting;
    use flowfine::config::VersionFormatting::Numeric;
    use flowfine::migration::parser::get_migrations;
    use flowfine::migration::version::MigrationVersionKey;
    use flowfine::migration::Migration;
    use flowfine::runner::{MigrationRunner, ScyllaMigrationRunner};
    use lazy_static::lazy_static;
    use rstest::{fixture, rstest};
    use scylla::{Session, SessionBuilder};
    use std::sync::Arc;

    lazy_static! {
        static ref KEYSPACE: &'static str = "flowfine";
        static ref PATH: &'static str = "./tests/data/int/numeric_migrations";
        static ref APPLY_MIGRATIONS_FAILED: &'static str = "Failed to apply migrations";
        static ref LOAD_MIGRATIONS_FAILED: &'static str = "Failed to load migrations files";
        static ref PARSE_MIGRATIONS_FAILED: &'static str = "Failed to parse migrations";
    }

    #[fixture]
    async fn session() -> Arc<Session> {
        let session = SessionBuilder::new()
            .known_node("[::1]:9042".to_string())
            .build()
            .await
            .unwrap();
        Arc::new(session)
    }

    #[fixture]
    async fn runner(#[future] session: Arc<Session>) -> ScyllaMigrationRunner {
        ScyllaMigrationRunner::new(session.await, *KEYSPACE)
    }

    async fn before_each(session: Arc<Session>) {
        let drop_query = format!("DROP KEYSPACE IF EXISTS {};", *KEYSPACE);
        let create_query = format!("CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor' : 1}};", *KEYSPACE);
        session
            .query(drop_query, &[])
            .await
            .expect("Failed to drop keyspace");
        session
            .query(create_query, &[])
            .await
            .expect("Failed to create keyspace");
    }

    #[rstest]
    #[tokio::test]
    async fn test_apply_empty_migration(
        #[future] session: Arc<Session>,
        #[future] runner: ScyllaMigrationRunner,
    ) {
        let session = session.await;
        let runner = runner.await;
        before_each(session).await;

        // given
        let migrations = Vec::new();

        // when
        let applied_migrations = runner
            .run(migrations)
            .await
            .expect(*APPLY_MIGRATIONS_FAILED);

        // then
        assert!(applied_migrations.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn test_apply_migrations(
        #[future] session: Arc<Session>,
        #[future] runner: ScyllaMigrationRunner,
    ) {
        let session = session.await;
        let runner = runner.await;
        before_each(session).await;

        // given
        let migrations = get_migrations(*PATH, &Numeric)
            .expect(*LOAD_MIGRATIONS_FAILED)
            .into_result()
            .expect(*PARSE_MIGRATIONS_FAILED);

        // when
        let applied_migrations = runner
            .run(migrations.clone())
            .await
            .expect(*APPLY_MIGRATIONS_FAILED);

        // then
        assert_eq!(migrations.len(), applied_migrations.len());
    }

    #[rstest]
    #[tokio::test]
    async fn test_ignore_already_applied_migrations(
        #[future] session: Arc<Session>,
        #[future] runner: ScyllaMigrationRunner,
    ) {
        let session = session.await;
        let runner = runner.await;
        before_each(session).await;

        // given
        let version_formatting = Numeric;
        let mut migrations = get_migrations(*PATH, &version_formatting)
            .expect(*LOAD_MIGRATIONS_FAILED)
            .into_result()
            .expect(*PARSE_MIGRATIONS_FAILED);
        let next_migration = new_migration(
            &version_formatting,
            "1.3",
            "delete_data",
            "DELETE FROM flowfine.test_table WHERE id = 1;",
        )
        .unwrap();

        // when
        runner
            .run(migrations.clone())
            .await
            .expect(*APPLY_MIGRATIONS_FAILED);

        migrations.push(next_migration.clone());
        let applied_migrations = runner
            .run(migrations.clone())
            .await
            .expect(*APPLY_MIGRATIONS_FAILED);

        // then
        assert_eq!(applied_migrations.len(), 1);
        assert_eq!(applied_migrations[0].version, next_migration.version);
        assert_eq!(applied_migrations[0].name, next_migration.name);
        assert_eq!(applied_migrations[0].filename, next_migration.filename);
        assert_eq!(applied_migrations[0].success, true);
    }

    fn new_migration(
        version_formatting: &VersionFormatting,
        version: &str,
        name: &str,
        content: &str,
    ) -> Option<Migration> {
        let version_key = MigrationVersionKey::new(version_formatting, version)?;
        let migration = Migration {
            filename: format!("V{}_{}.cql", version, name),
            version: version.to_string(),
            version_key,
            name: name.to_string(),
            content: content.to_string(),
            queries: vec![content.to_string()],
        };

        Some(migration)
    }
}

#[cfg(test)]
mod tests {
    use flowfine::config::VersionFormatting::Numeric;
    use flowfine::migration::parser::get_migrations;
    use flowfine::runner::{MigrationRunner, ScyllaMigrationRunner};
    use lazy_static::lazy_static;
    use rstest::{fixture, rstest};
    use scylla::{Session, SessionBuilder};
    use std::sync::Arc;

    lazy_static! {
        static ref KEYSPACE: &'static str = "flowfine";
        static ref PATH: &'static str = "./tests/data/int/numeric_migrations";
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
        let _ = session.query(drop_query, &[]).await;
        let _ = session.query(create_query, &[]).await;
    }

    #[rstest]
    #[tokio::test]
    async fn test_apply_empty_migration(
        #[future] session: Arc<Session>,
        #[future] runner: ScyllaMigrationRunner,
    ) {
        before_each(session.await).await;

        // given
        let runner = runner.await;
        let migrations = Vec::new();

        // when
        let applied_migrations = runner.run(migrations).await.unwrap();

        // then
        assert!(applied_migrations.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn test_apply_migrations(
        #[future] session: Arc<Session>,
        #[future] runner: ScyllaMigrationRunner,
    ) {
        before_each(session.await).await;

        // given
        let runner = runner.await;
        let migrations = get_migrations(*PATH, &Numeric)
            .unwrap()
            .into_result()
            .unwrap();

        // when
        let applied_migrations = runner.run(migrations.clone()).await.unwrap();

        // then
        assert_eq!(migrations.len(), applied_migrations.len());
    }
}

#[cfg(test)]
mod tests {
    use flowfine::config::VersionFormatting::Numeric;
    use flowfine::migration::parser::get_migrations;
    use flowfine::runner::{MigrationRunner, ScyllaMigrationRunner};
    use rstest::{fixture, rstest};
    use scylla::{Session, SessionBuilder};
    use std::sync::Arc;

    const KEYSPACE: &str = "flowfine";
    const PATH: &str = "./tests/data/int/numeric_migrations";

    #[fixture]
    async fn session() -> Arc<Session> {
        let session = SessionBuilder::new()
            .known_node("[::1]:9042".to_string())
            .use_keyspace(KEYSPACE, false)
            .build()
            .await
            .unwrap();
        Arc::new(session)
    }

    #[fixture]
    async fn runner(#[future] session: Arc<Session>) -> ScyllaMigrationRunner {
        ScyllaMigrationRunner::new(session.await, KEYSPACE)
    }

    #[ignore]
    #[rstest]
    #[tokio::test]
    async fn test_apply_empty_migration(#[future] runner: ScyllaMigrationRunner) {
        // given
        let runner = runner.await;
        let migrations = Vec::new();

        // when
        let applied_migrations = runner.run(migrations).await.unwrap();

        // then
        assert!(applied_migrations.is_empty());
    }

    #[ignore]
    #[rstest]
    #[tokio::test]
    async fn test_apply_migrations(#[future] runner: ScyllaMigrationRunner) {
        // given
        let runner = runner.await;
        let migrations = get_migrations(PATH, &Numeric)
            .unwrap()
            .into_result()
            .unwrap();

        // when
        let applied_migrations = runner.run(migrations.clone()).await.unwrap();

        // then
        assert_eq!(migrations.len(), applied_migrations.len());
    }
}

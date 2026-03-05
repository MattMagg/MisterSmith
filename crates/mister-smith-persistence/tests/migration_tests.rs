//! Env-gated migration integration tests.
//!
//! These tests require a running PostgreSQL instance and `DATABASE_URL` env var.
//! Run with: `DATABASE_URL=postgres://... cargo test -p mister-smith-persistence --test migration_tests -- --ignored`

use mister_smith_persistence::postgres::migrations::MigrationRunner;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

/// Create a fresh PostgreSQL pool for testing.
async fn setup_pool() -> sqlx::PgPool {
    let url = database_url().expect("DATABASE_URL required");
    sqlx::PgPool::connect(&url)
        .await
        .expect("Failed to connect to PostgreSQL")
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn migration_runs_cleanly() {
    let pool = setup_pool().await;
    let runner = MigrationRunner::new(pool);

    let applied = runner.run().await.expect("Migrations should succeed");

    // At least our 4 migrations should exist (may be 0 if already applied)
    let version = runner
        .current_version()
        .await
        .expect("current_version should succeed");
    assert!(version.is_some(), "Should have at least one applied migration");
    assert!(
        version.unwrap() > 0,
        "Latest version should be greater than 0"
    );

    // Verify all migrations are applied
    assert!(
        runner.verify().await.expect("verify should succeed"),
        "All migrations should be verified after run"
    );

    // applied count should be consistent
    let _ = applied; // may be 0 on re-run — that's fine (idempotent)
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn migration_status_lists_all() {
    let pool = setup_pool().await;
    let runner = MigrationRunner::new(pool);

    // Ensure migrations are applied first
    runner.run().await.expect("Migrations should succeed");

    let statuses = runner.status().await.expect("status should succeed");
    assert!(
        statuses.len() >= 4,
        "Should have at least 4 migrations, got {}",
        statuses.len()
    );

    // All should be applied
    for s in &statuses {
        assert!(
            s.applied,
            "Migration {} ({}) should be applied",
            s.version, s.description
        );
        assert!(
            !s.checksum.is_empty(),
            "Migration {} should have a checksum",
            s.version
        );
    }

    // Versions should be in ascending order
    for window in statuses.windows(2) {
        assert!(
            window[0].version < window[1].version,
            "Migrations should be in ascending version order"
        );
    }
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn migration_verify_returns_true() {
    let pool = setup_pool().await;
    let runner = MigrationRunner::new(pool);

    // Ensure migrations are applied
    runner.run().await.expect("Migrations should succeed");

    let verified = runner.verify().await.expect("verify should succeed");
    assert!(verified, "verify() should return true after all migrations applied");
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn migration_is_idempotent() {
    let pool = setup_pool().await;
    let runner = MigrationRunner::new(pool);

    // Run migrations twice
    let first_run = runner.run().await.expect("First run should succeed");
    let second_run = runner.run().await.expect("Second run should succeed");

    // Second run should apply zero new migrations
    assert_eq!(
        second_run, 0,
        "Second run should be a no-op, but applied {} migrations",
        second_run
    );

    // State should be consistent
    assert!(
        runner.verify().await.expect("verify should succeed"),
        "verify() should return true after idempotent run"
    );

    let _ = first_run; // first run count depends on initial state
}

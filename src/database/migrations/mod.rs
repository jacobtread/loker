use std::ops::DerefMut;

use crate::database::{DbExecutor, DbResult, DbTransaction};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::prelude::FromRow;

pub const MIGRATIONS: &[(&str, &str)] = &[(
    "m1_create_secrets_tables",
    include_str!("./m1_create_secrets_tables.sql"),
)];

const MIGRATIONS_SETUP_SQL: &str = include_str!("./m0_create_migrations_table.sql");

/// Structure for tracking migrations applied to the root
#[derive(Debug, Clone, FromRow, Serialize)]
struct Migration {
    name: String,
    applied_at: DateTime<Utc>,
}

struct CreateMigration {
    name: String,
    applied_at: DateTime<Utc>,
}

/// Create a new tenant migration
async fn create_migration(db: impl DbExecutor<'_>, create: CreateMigration) -> DbResult<()> {
    sqlx::query(
        r#"
        INSERT INTO "migrations" ("name", "applied_at")
        VALUES (?, ?)
    "#,
    )
    .bind(create.name)
    .bind(create.applied_at)
    .execute(db)
    .await?;

    Ok(())
}

/// Find all applied migrations
async fn applied_migrations(db: impl DbExecutor<'_>) -> DbResult<Vec<Migration>> {
    sqlx::query_as(r#"SELECT * FROM "migrations""#)
        .fetch_all(db)
        .await
}

pub async fn setup_migrations(t: &mut DbTransaction<'_>) -> DbResult<()> {
    apply_migration(t, "m0_create_migrations_table", MIGRATIONS_SETUP_SQL).await?;
    Ok(())
}

pub async fn apply_migrations(t: &mut DbTransaction<'_>) -> DbResult<()> {
    let migrations = applied_migrations(t.deref_mut()).await?;

    for (migration_name, migration) in MIGRATIONS {
        // Skip already applied migrations
        if migrations
            .iter()
            .any(|migration| migration.name.eq(migration_name))
        {
            continue;
        }

        // Apply the migration
        apply_migration(t, migration_name, migration).await?;

        // Store the applied migration
        create_migration(
            t.deref_mut(),
            CreateMigration {
                name: migration_name.to_string(),
                applied_at: Utc::now(),
            },
        )
        .await?;
    }

    Ok(())
}

/// Apply a migration to the specific database
pub async fn apply_migration(
    db: &mut DbTransaction<'_>,
    migration_name: &str,
    migration: &str,
) -> DbResult<()> {
    // Split the SQL queries into multiple queries
    let queries = migration
        .split(';')
        .map(|query| query.trim())
        .filter(|query| !query.is_empty());

    for query in queries {
        let result = sqlx::query(query)
            .execute(db.deref_mut())
            .await
            .inspect_err(|error| {
                tracing::error!(?error, ?migration_name, "failed to perform migration")
            })?;
        let rows_affected = result.rows_affected();

        tracing::debug!(?migration_name, ?rows_affected, "applied migration query");
    }

    Ok(())
}

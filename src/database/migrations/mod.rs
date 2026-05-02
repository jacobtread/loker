use crate::database::DbResult;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::Serialize;
use tokio_rusqlite::{
    Row, params,
    rusqlite::{self, Connection},
};

pub const MIGRATIONS: &[(&str, &str)] = &[(
    "m1_create_secrets_tables",
    include_str!("./m1_create_secrets_tables.sql"),
)];

const MIGRATIONS_SETUP_SQL: &str = include_str!("./m0_create_migrations_table.sql");

/// Structure for tracking migrations applied to the root
#[derive(Debug, Clone, Serialize)]
struct Migration {
    name: String,
    applied_at: DateTime<Utc>,
}

impl<'a> TryFrom<&'a Row<'a>> for Migration {
    type Error = rusqlite::Error;

    fn try_from(value: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.get("name")?,
            applied_at: value.get("applied_at")?,
        })
    }
}

struct CreateMigration {
    name: String,
    applied_at: DateTime<Utc>,
}

/// Create a new tenant migration
fn create_migration(db: &Connection, create: CreateMigration) -> DbResult<()> {
    db.execute(
        r#"
        INSERT INTO "migrations" ("name", "applied_at")
        VALUES (?1, ?2)
    "#,
        params![create.name, create.applied_at],
    )?;
    Ok(())
}

/// Find all applied migrations
fn applied_migrations(db: &Connection) -> DbResult<Vec<Migration>> {
    db.prepare(r#"SELECT * FROM "migrations""#)?
        .query_map(params![], |row| Migration::try_from(row))?
        .try_collect()
}

pub fn setup_migrations(t: &Connection) -> DbResult<()> {
    apply_migration(t, "m0_create_migrations_table", MIGRATIONS_SETUP_SQL)?;
    Ok(())
}

pub fn apply_migrations(t: &Connection) -> DbResult<()> {
    let migrations = applied_migrations(t)?;

    for (migration_name, migration) in MIGRATIONS {
        // Skip already applied migrations
        if migrations
            .iter()
            .any(|migration| migration.name.eq(migration_name))
        {
            continue;
        }

        // Apply the migration
        apply_migration(t, migration_name, migration)?;

        // Store the applied migration
        create_migration(
            t,
            CreateMigration {
                name: migration_name.to_string(),
                applied_at: Utc::now(),
            },
        )?;
    }

    Ok(())
}

/// Apply a migration to the specific database
pub fn apply_migration(db: &Connection, migration_name: &str, migration: &str) -> DbResult<()> {
    // Split the SQL queries into multiple queries
    let queries = migration
        .split(';')
        .map(|query| query.trim())
        .filter(|query| !query.is_empty());

    for query in queries {
        let rows_affected = db.execute(query, params![]).inspect_err(|error| {
            tracing::error!(?error, ?migration_name, "failed to perform migration")
        })?;

        tracing::debug!(?migration_name, ?rows_affected, "applied migration query");
    }

    Ok(())
}

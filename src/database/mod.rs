use std::{ops::DerefMut, path::Path};

use futures::future::BoxFuture;
use sqlx::{Sqlite, SqlitePool, Transaction, sqlite::SqlitePoolOptions};

pub use sqlx::SqliteExecutor as DbExecutor;
use thiserror::Error;
use tokio::fs::File;

use crate::database::migrations::{apply_migrations, setup_migrations};

pub mod migrations;
pub mod secrets;

/// Type of the database connection pool
pub type DbPool = SqlitePool;

/// Short type alias for a database error
pub type DbErr = sqlx::Error;

/// Type alias for a result where the error is a [DbErr]
pub type DbResult<T> = Result<T, DbErr>;

/// Type of a database transaction
pub type DbTransaction<'c> = Transaction<'c, Sqlite>;

#[derive(Debug, Error)]
pub enum CreateDatabaseError {
    #[error("failed to create database file parent folders")]
    CreateParentFolders(std::io::Error),

    #[error("failed to create database file")]
    CreateFile(std::io::Error),

    #[error(transparent)]
    Db(#[from] DbErr),
}

pub async fn create_database(key: String, raw_path: String) -> Result<DbPool, CreateDatabaseError> {
    let path = Path::new(&raw_path);
    if !path.exists() {
        // Ensure the path to the database exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(CreateDatabaseError::CreateParentFolders)?;
        }

        let _file = File::create(&path)
            .await
            .map_err(CreateDatabaseError::CreateFile)?;
    }

    let pool = SqlitePoolOptions::new()
        .after_connect(move |mut connection, _metadata| {
            let key = key.clone();
            Box::pin(async move {
                // Set database encryption key
                sqlx::query(&format!("PRAGMA key = '{key}';"))
                    .execute(connection.deref_mut())
                    .await?;

                // Enable case sensitive LIKE
                sqlx::query("PRAGMA case_sensitive_like = ON;")
                    .execute(connection)
                    .await?;

                Ok(())
            })
        })
        .connect(&format!("sqlite:{raw_path}"))
        .await?;

    initialize_database(&pool).await?;

    Ok(pool)
}

/// Initializes the database ensuring the migrations table is setup and that all migrations
/// are applied
pub async fn initialize_database(db: &DbPool) -> DbResult<()> {
    transaction(db, move |t| {
        Box::pin(async move {
            setup_migrations(t).await?;
            apply_migrations(t).await?;
            Ok::<_, DbErr>(())
        })
    })
    .await?;

    Ok(())
}

/// Helper to perform an action future that requires a database transaction
/// ensures that if the action fails the database will rollback immediately
pub async fn transaction<'db, A, O, E>(db: &'db DbPool, action: A) -> Result<O, E>
where
    A: for<'a> FnOnce(&'a mut DbTransaction<'db>) -> BoxFuture<'a, Result<O, E>>,
    E: From<DbErr>,
{
    let mut t = db
        .begin()
        .await
        .inspect_err(|error| tracing::error!(?error, "failed to begin transaction"))?;

    let output = match action(&mut t).await {
        Ok(value) => value,
        Err(error) => {
            if let Err(error) = t.rollback().await {
                tracing::error!(?error, "failed to rollback transaction")
            }

            return Err(error);
        }
    };

    t.commit()
        .await
        .inspect_err(|error| tracing::error!(?error, "failed to commit transaction"))?;

    Ok(output)
}

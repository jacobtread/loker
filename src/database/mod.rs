use std::path::Path;

use thiserror::Error;
use tokio::fs::File;
use tokio_rusqlite::{Connection, rusqlite};

use crate::database::migrations::{apply_migrations, setup_migrations};

pub mod ext;
pub mod migrations;
pub mod secrets;

pub type DbHandle = Connection;

pub type DbConnection = rusqlite::Connection;

/// Short type alias for a database error
pub type DbErr = rusqlite::Error;

/// Type alias for a result where the error is a [DbErr]
pub type DbResult<T> = Result<T, DbErr>;

#[derive(Debug, Error)]
pub enum CreateDatabaseError {
    #[error("failed to create database file parent folders")]
    CreateParentFolders(std::io::Error),

    #[error("failed to create database file")]
    CreateFile(std::io::Error),

    #[error(transparent)]
    AsyncDb(#[from] tokio_rusqlite::Error),

    #[error(transparent)]
    Db(#[from] rusqlite::Error),
}

pub async fn create_database(
    key: String,
    raw_path: String,
) -> Result<Connection, CreateDatabaseError> {
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

    let db = Connection::open(path).await?;
    db.call(move |db| {
        db.pragma_update(None, "key", key)?;
        db.pragma_update(None, "case_sensitive_like", true)?;
        initialize_database(db)?;
        Ok(())
    })
    .await?;

    Ok(db)
}

/// Initializes the database ensuring the migrations table is setup and that all migrations
/// are applied
pub fn initialize_database(db: &mut rusqlite::Connection) -> DbResult<()> {
    transaction(db, move |t| {
        setup_migrations(t)?;
        apply_migrations(t)?;
        Ok::<_, DbErr>(())
    })?;

    Ok(())
}

/// Helper to perform an action future that requires a database transaction
/// ensures that if the action fails the database will rollback immediately
pub fn transaction<A, O, E>(db: &mut rusqlite::Connection, action: A) -> Result<O, E>
where
    A: for<'a> FnOnce(&rusqlite::Connection) -> Result<O, E>,
    E: From<DbErr>,
{
    let mut t = db
        .transaction()
        .inspect_err(|error| tracing::error!(?error, "failed to begin transaction"))?;

    let output = match action(&mut t) {
        Ok(value) => value,
        Err(error) => {
            if let Err(error) = t.rollback() {
                tracing::error!(?error, "failed to rollback transaction")
            }

            return Err(error);
        }
    };

    t.commit()
        .inspect_err(|error| tracing::error!(?error, "failed to commit transaction"))?;

    Ok(output)
}

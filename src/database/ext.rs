use serde::de::DeserializeOwned;
use tokio_rusqlite::{ErrorCode, Row, RowIndex, rusqlite, types::FromSqlError};

pub trait RowExt {
    fn get_json<I: RowIndex, T: DeserializeOwned>(&self, idx: I) -> Result<T, rusqlite::Error>;
}

impl<'a> RowExt for Row<'a> {
    fn get_json<I: RowIndex, T: DeserializeOwned>(&self, idx: I) -> Result<T, rusqlite::Error> {
        let value: serde_json::Value = self.get(idx)?;
        let parsed_value: T = serde_json::from_value(value).map_err(FromSqlError::other)?;

        Ok(parsed_value)
    }
}

pub trait SqlErrorExt {
    fn is_constraint_violation(&self) -> bool;
}

impl SqlErrorExt for rusqlite::Error {
    fn is_constraint_violation(&self) -> bool {
        self.sqlite_error_code()
            .is_some_and(|code| matches!(code, ErrorCode::ConstraintViolation))
    }
}

use crate::{
    database::{DbResult, ext::RowExt},
    handlers::models::Filter,
    utils::filter::split_search_terms,
};
use chrono::{DateTime, Days, Utc};
use itertools::Itertools;
use serde::Deserialize;
use tokio_rusqlite::{
    OptionalExtension, Row, ToSql, params, params_from_iter,
    rusqlite::{self, Connection},
    types::FromSqlError,
};

#[derive(Clone)]
pub struct StoredSecret {
    pub arn: String,
    pub name: String,
    //
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub scheduled_delete_at: Option<DateTime<Utc>>,
    //
    pub version_id: String,
    pub version_stages: Vec<String>,
    //
    pub description: Option<String>,
    pub secret_string: Option<String>,
    pub secret_binary: Option<String>,
    //
    pub version_created_at: DateTime<Utc>,
    pub version_last_accessed_at: Option<DateTime<Utc>>,
    //
    pub version_tags: Vec<StoredVersionTags>,
}

impl StoredSecret {
    pub fn is_value_eq(
        &self,
        secret_string: &Option<String>,
        secret_binary: &Option<String>,
    ) -> bool {
        self.secret_string.eq(secret_string) && self.secret_binary.eq(secret_binary)
    }
}

impl<'a> TryFrom<&'a Row<'a>> for StoredSecret {
    type Error = rusqlite::Error;

    fn try_from(value: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            arn: value.get("arn")?,
            name: value.get("name")?,
            created_at: value.get("created_at")?,
            updated_at: value.get("updated_at")?,
            deleted_at: value.get("deleted_at")?,
            scheduled_delete_at: value.get("scheduled_delete_at")?,
            version_id: value.get("version_id")?,
            version_stages: value.get_json("version_stages")?,
            description: value.get("description")?,
            secret_string: value.get("secret_string")?,
            secret_binary: value.get("secret_binary")?,
            version_created_at: value.get("version_created_at")?,
            version_last_accessed_at: value.get("Version_last_accessed_at")?,
            version_tags: value.get_json("version_tags")?,
        })
    }
}

#[derive(Clone)]
pub struct StoredSecretWithVersionStages {
    pub arn: String,
    pub name: String,
    //
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub scheduled_delete_at: Option<DateTime<Utc>>,
    //
    pub version_id: String,
    pub version_stages: Vec<String>,
    //
    pub description: Option<String>,
    pub secret_string: Option<String>,
    pub secret_binary: Option<String>,
    //
    pub version_created_at: DateTime<Utc>,
    pub version_last_accessed_at: Option<DateTime<Utc>>,
    //
    pub version_tags: Vec<StoredVersionTags>,
    //
    pub versions: Vec<StoredVersionsListItem>,
}

impl<'a> TryFrom<&'a Row<'a>> for StoredSecretWithVersionStages {
    type Error = rusqlite::Error;

    fn try_from(value: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            arn: value.get("arn")?,
            name: value.get("name")?,
            created_at: value.get("created_at")?,
            updated_at: value.get("updated_at")?,
            deleted_at: value.get("deleted_at")?,
            scheduled_delete_at: value.get("scheduled_delete_at")?,
            version_id: value.get("version_id")?,
            version_stages: value.get_json("version_stages")?,
            description: value.get("description")?,
            secret_string: value.get("secret_string")?,
            secret_binary: value.get("secret_binary")?,
            version_created_at: value.get("version_created_at")?,
            version_last_accessed_at: value.get("Version_last_accessed_at")?,
            version_tags: value.get_json("version_tags")?,
            versions: value.get_json("versions")?,
        })
    }
}

#[derive(Clone, Deserialize)]
pub struct StoredVersionsListItem {
    pub version_id: String,
    pub version_stages: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct SecretVersion {
    pub secret_arn: String,
    //
    pub version_id: String,
    pub version_stages: Vec<String>,
    //
    pub secret_string: Option<String>,
    pub secret_binary: Option<String>,
    //
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
}

impl<'a> TryFrom<&'a Row<'a>> for SecretVersion {
    type Error = rusqlite::Error;

    fn try_from(value: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            secret_arn: value.get("secret_arn")?,
            version_id: value.get("version_id")?,
            version_stages: value.get_json("version_stages")?,
            secret_string: value.get("secret_string")?,
            secret_binary: value.get("secret_binary")?,
            created_at: value.get("created_at")?,
            last_accessed_at: value.get("last_accessed_at")?,
        })
    }
}

#[derive(Clone, Deserialize)]
pub struct StoredVersionTags {
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct CreateSecret {
    pub arn: String,
    pub name: String,
    pub description: Option<String>,
}

/// Create a new "secret" with no versions
pub fn create_secret(db: &Connection, create: CreateSecret) -> DbResult<()> {
    let created_at = Utc::now();

    db.execute(
        r#"
        INSERT INTO "secrets" ("arn", "name", "description", "created_at") VALUES (?, ?, ?, ?)
    "#,
        params![create.arn, create.name, create.description, created_at],
    )?;

    Ok(())
}

/// Updates the description of a secret
pub fn update_secret_description(db: &Connection, arn: &str, description: &str) -> DbResult<usize> {
    let updated_at = Utc::now();

    db.execute(
        r#"UPDATE "secrets" SET "description" = ?, "updated_at" = ? WHERE "secrets"."arn" = ?"#,
        params![description, updated_at, arn],
    )
}

/// Remove a secret
pub fn delete_secret(db: &Connection, secret_arn: &str) -> DbResult<usize> {
    db.execute(
        r#"DELETE FROM "secrets" WHERE "arn" = ?"#,
        params![secret_arn],
    )
}

/// Get the ARN's of all the secrets that are scheduled for deletion
///
/// Not used by the actual application, only used within tests to ensure
/// a deletion was properly scheduled
pub fn get_scheduled_secret_deletions(db: &Connection) -> DbResult<Vec<String>> {
    db.prepare(r#"SELECT "arn" FROM "secrets" WHERE "scheduled_delete_at" IS NOT NULL"#)?
        .query_map(params![], |row| row.get::<_, String>(0))?
        .try_collect()
}

/// Delete all secrets that have past their "scheduled_delete_at" date
/// deletes anything where the scheduled_delete_at date is less than `before`
pub fn delete_scheduled_secrets(db: &Connection, before: DateTime<Utc>) -> DbResult<usize> {
    db.execute(
        r#"DELETE FROM "secrets" WHERE "scheduled_delete_at" < ?"#,
        params![before],
    )
}

/// Mark a secret for deletion, sets the scheduled deletion date for `days` days
/// into the future
pub fn schedule_delete_secret(
    db: &Connection,
    secret_arn: &str,
    days: i32,
) -> DbResult<DateTime<Utc>> {
    let deleted_at = Utc::now();
    let scheduled_deleted_at = deleted_at
        .checked_add_days(Days::new(days as u64))
        .ok_or_else(|| {
            FromSqlError::other(Box::new(std::io::Error::other(
                "failed to create a future timestamp",
            )))
        })?;

    db.query_one(
        r#"
        UPDATE "secrets"
        SET
            "deleted_at" = ?,
            "scheduled_delete_at" = ?
        WHERE "arn" = ?
        RETURNING "scheduled_delete_at"
        "#,
        params![deleted_at, scheduled_deleted_at, secret_arn],
        |row| row.get::<_, DateTime<Utc>>(0),
    )
}

/// Cancel a secrets deletion
pub fn cancel_delete_secret(db: &Connection, secret_arn: &str) -> DbResult<()> {
    db.execute(
        r#"
        UPDATE "secrets"
        SET
            "deleted_at" = NULL,
            "scheduled_delete_at" = NULL
        WHERE "arn" = ?
        "#,
        params![secret_arn],
    )?;

    Ok(())
}

/// Set a tag on a secret
pub fn put_secret_tag(db: &Connection, secret_arn: &str, key: &str, value: &str) -> DbResult<()> {
    let now = Utc::now();

    db.execute(
        r#"
        INSERT INTO "secrets_tags" ("secret_arn", "key", "value", "created_at")
        VALUES (?, ?, ?, ?)
        ON CONFLICT("secret_arn", "key")
        DO UPDATE SET
            "value" = "excluded"."value",
            "updated_at" = "excluded"."created_at"
        "#,
        params![secret_arn, key, value, now],
    )?;

    Ok(())
}

/// Remove a tag from a secret
pub fn remove_secret_tag(db: &Connection, secret_arn: &str, key: &str) -> DbResult<usize> {
    db.execute(
        r#"DELETE FROM "secrets_tags" WHERE "secret_arn" = ? AND "key" = ?"#,
        params![secret_arn, key],
    )
}

pub struct CreateSecretVersion {
    pub secret_arn: String,
    pub version_id: String,
    //
    pub secret_string: Option<String>,
    pub secret_binary: Option<String>,
}

/// Creates a new version of a secret
pub fn create_secret_version(db: &Connection, create: CreateSecretVersion) -> DbResult<()> {
    let now = Utc::now();

    db.execute(
        r#"
        INSERT INTO "secrets_versions" ("secret_arn", "version_id", "secret_string", "secret_binary", "created_at")
        VALUES (?, ?, ?, ?, ?)
        "#,
        params![create.secret_arn, create.version_id, create.secret_string, create.secret_binary, now]
    )?;

    Ok(())
}

/// Updates the last access date of a secret version
pub fn update_secret_version_last_accessed(
    db: &Connection,
    secret_arn: &str,
    version_id: &str,
) -> DbResult<()> {
    let now = Utc::now();

    db.execute(
        r#"
        UPDATE "secrets_versions"
        SET "last_accessed_at" = ?
        WHERE "secret_arn" = ? AND "version_id" = ?"#,
        params![now, secret_arn, version_id],
    )?;

    Ok(())
}

/// Add a secret version stage to a specific secret version
pub fn add_secret_version_stage(
    db: &Connection,
    secret_arn: &str,
    version_id: &str,
    version_stage: &str,
) -> DbResult<()> {
    let created_at = Utc::now();
    db.execute(
        r#"
        INSERT INTO "secret_version_stages" ("secret_arn", "version_id", "value", "created_at")
        VALUES (?, ?, ?, ?)
    "#,
        params![secret_arn, version_id, version_stage, created_at],
    )?;

    Ok(())
}

/// Remove a secret version stage from a specific secret version
pub fn remove_secret_version_stage(
    db: &Connection,
    secret_arn: &str,
    version_id: &str,
    version_stage: &str,
) -> DbResult<usize> {
    db.execute(
        r#"
        DELETE FROM "secret_version_stages"
        WHERE "secret_arn" = ? AND "version_id" = ? AND "value" = ?
    "#,
        params![secret_arn, version_id, version_stage],
    )
}

/// Remove a version stage label from any version in a secret
pub fn remove_secret_version_stage_any(
    db: &Connection,
    secret_arn: &str,
    version_stage: &str,
) -> DbResult<usize> {
    db.execute(
        r#"
        DELETE FROM "secret_version_stages"
        WHERE "secret_arn" = ? AND "value" = ?
    "#,
        params![secret_arn, version_stage],
    )
}

/// Get the current version of a secret where the name OR arn matches the `secret_id`
pub fn get_secret_latest_version(
    db: &Connection,
    secret_id: &str,
) -> DbResult<Option<StoredSecret>> {
    get_secret_by_version_stage(db, secret_id, "AWSCURRENT")
}

/// Check if the value is a partial arn
fn is_partial_arn(value: &str) -> bool {
    let is_arn_like = value.starts_with("arn:") && value.split(':').count() >= 6;
    is_arn_like && (value.contains('*') || value.contains('?'))
}

/// Create a partial arn acceptable for a LIKE query
fn make_partial_arn_like_query(value: &str) -> Option<String> {
    if is_partial_arn(value) {
        Some(
            value
                // Escape underscores
                .replace('_', r"\_")
                // Replace the placeholders with SQLite LIKE equivalents
                .replace('*', "%")
                .replace('?', "_"),
        )
    } else {
        None
    }
}

/// Get a secret where the name OR arn matches the `secret_id` and there is a version
/// with the version ID of `version_id`
pub fn get_secret_by_version_id(
    db: &Connection,
    secret_id: &str,
    version_id: &str,
) -> DbResult<Option<StoredSecret>> {
    let partial_arn = make_partial_arn_like_query(secret_id);

    db.query_one(
        r#"
        SELECT
            "secret".*,
            "secret_version"."version_id",
            "secret_version"."secret_string",
            "secret_version"."secret_binary",
            "secret_version"."created_at" AS "version_created_at",
            "secret_version"."last_accessed_at" AS "version_last_accessed_at",
            COALESCE((
                SELECT json_group_array("version_stage"."value")
                FROM "secret_version_stages" "version_stage"
                WHERE "version_stage"."secret_arn" = "secret_version"."secret_arn"
                    AND "version_stage"."version_id" = "secret_version"."version_id"
            ), '[]') AS "version_stages",
            COALESCE((
                SELECT json_group_array(
                    json_object(
                        'key', "secret_tag"."key",
                        'value', "secret_tag"."value",
                        'created_at', "secret_tag"."created_at",
                        'updated_at', "secret_tag"."updated_at"
                    )
                )
                FROM "secrets_tags" "secret_tag"
                WHERE "secret_tag"."secret_arn" = "secret"."arn"
            ), '[]') AS "version_tags"
        FROM "secrets" "secret"
        JOIN "secrets_versions" "secret_version"
            ON "secret_version"."secret_arn" = "secret"."arn"
            AND "secret_version"."version_id" = ?
        WHERE "secret"."name" = ? OR "secret"."arn" = ?
            OR (? = TRUE AND "secret"."arn" LIKE ?)
        LIMIT 1;
    "#,
        params![
            version_id,
            secret_id,
            secret_id,
            partial_arn.is_some(),
            partial_arn
        ],
        |row| StoredSecret::try_from(row),
    )
    .optional()
}

/// Get a secret where the name OR arn matches the `secret_id` and there is a version
/// in `version_stage`
pub fn get_secret_by_version_stage(
    db: &Connection,
    secret_id: &str,
    version_stage: &str,
) -> DbResult<Option<StoredSecret>> {
    let partial_arn = make_partial_arn_like_query(secret_id);

    db.query_one(
        r#"
        SELECT
            "secret".*,
            "secret_version"."version_id",
            "secret_version"."secret_string",
            "secret_version"."secret_binary",
            "secret_version"."created_at" AS "version_created_at",
            "secret_version"."last_accessed_at" AS "version_last_accessed_at",
            COALESCE((
                SELECT json_group_array("version_stage"."value")
                FROM "secret_version_stages" "version_stage"
                WHERE "version_stage"."secret_arn" = "secret_version"."secret_arn"
                    AND "version_stage"."version_id" = "secret_version"."version_id"
            ), '[]') AS "version_stages",
            COALESCE((
                SELECT json_group_array(
                    json_object(
                        'key', "secret_tag"."key",
                        'value', "secret_tag"."value",
                        'created_at', "secret_tag"."created_at",
                        'updated_at', "secret_tag"."updated_at"
                    )
                )
                FROM "secrets_tags" "secret_tag"
                WHERE "secret_tag"."secret_arn" = "secret"."arn"
            ), '[]') AS "version_tags"
        FROM "secrets" "secret"
        JOIN "secrets_versions" "secret_version"
            ON "secret_version"."secret_arn" = "secret"."arn"
        JOIN "secret_version_stages" "version_stage"
            ON "version_stage"."secret_arn" = "secret_version"."secret_arn"
            AND "version_stage"."version_id" = "secret_version"."version_id"
            AND "version_stage"."value" = ?
        WHERE "secret"."name" = ? OR "secret"."arn" = ?
            OR (? = TRUE AND "secret"."arn" LIKE ?)
        ORDER BY "secret_version"."created_at" DESC
        LIMIT 1;
    "#,
        params![
            version_stage,
            secret_id,
            secret_id,
            partial_arn.is_some(),
            partial_arn
        ],
        |row| StoredSecret::try_from(row),
    )
    .optional()
}

/// Get a secret where the name OR arn matches the `secret_id` and there is a version
/// in `version_stage` with the version ID `version_id`
pub fn get_secret_by_version_stage_and_id(
    db: &Connection,
    secret_id: &str,
    version_id: &str,
    version_stage: &str,
) -> DbResult<Option<StoredSecret>> {
    let partial_arn = make_partial_arn_like_query(secret_id);

    db.query_one(
        r#"
        SELECT
            "secret".*,
            "secret_version"."version_id",
            "secret_version"."secret_string",
            "secret_version"."secret_binary",
            "secret_version"."created_at" AS "version_created_at",
            "secret_version"."last_accessed_at" AS "version_last_accessed_at",
            COALESCE((
                SELECT json_group_array("version_stage"."value")
                FROM "secret_version_stages" "version_stage"
                WHERE "version_stage"."secret_arn" = "secret_version"."secret_arn"
                    AND "version_stage"."version_id" = "secret_version"."version_id"
            ), '[]') AS "version_stages",
            COALESCE((
                SELECT json_group_array(
                    json_object(
                        'key', "secret_tag"."key",
                        'value', "secret_tag"."value",
                        'created_at', "secret_tag"."created_at",
                        'updated_at', "secret_tag"."updated_at"
                    )
                )
                FROM "secrets_tags" "secret_tag"
                WHERE "secret_tag"."secret_arn" = "secret"."arn"
            ), '[]') AS "version_tags"
        FROM "secrets" "secret"
        JOIN ("secrets_versions" "secret_version" ON "secret_version"."secret_arn" = "secret"."arn")
            AND "secret_version"."version_id" = ?
        JOIN "secret_version_stages" "version_stage"
            ON "version_stage"."secret_arn" = "secret_version"."secret_arn"
            AND "version_stage"."version_id" = "secret_version"."version_id"
            AND "version_stage"."value" = ?
        WHERE "secret"."name" = ? OR "secret"."arn" = ?
            OR (? = TRUE AND "secret"."arn" LIKE ?)
        LIMIT 1;
    "#,
        params![
            version_id,
            version_stage,
            secret_id,
            secret_id,
            partial_arn.is_some(),
            partial_arn
        ],
        |row| StoredSecret::try_from(row),
    )
    .optional()
}

/// Generates the WHERE portion of a filtered query appending it to `query` returning
/// a list of parameters that need to be bound to the query
///
/// Assumes `query` has already specified the WHERE and at least one clause like 1=1
fn push_secret_filter_where(filters: &[Filter], query: &mut String) -> Vec<String> {
    let mut bound_values: Vec<String> = Vec::new();

    fn write_condition_cs<'a, 'b>(
        query: &mut String,
        bound_values: &mut Vec<String>,
        column: &str,
        values: impl IntoIterator<Item = &'a String>,
    ) {
        for (index, value) in values.into_iter().enumerate() {
            if index > 0 {
                query.push_str(" OR ");
            }

            // Escape underscores
            let value = value.replace('_', r"\_");

            query.push_str(column);

            // The ! prefix will invert the like
            match value.strip_prefix("!") {
                Some(value) => {
                    query.push_str(r" NOT LIKE ? ESCAPE '\'");
                    bound_values.push(format!("{value}%"));
                }
                _ => {
                    query.push_str(r" LIKE ? ESCAPE '\'");
                    bound_values.push(format!("{value}%"));
                }
            }
        }
    }

    fn write_condition_ci<'a, 'b>(
        query: &mut String,
        bound_values: &mut Vec<String>,
        column: &str,
        all: impl IntoIterator<Item = &'b String>,
    ) {
        for (index, value) in all.into_iter().enumerate() {
            if index > 0 {
                query.push_str(" OR ");
            }

            // Escape underscores
            let value = value.replace('_', r"\_");

            query.push_str(column);

            // The ! prefix will invert the like
            match value.strip_prefix("!") {
                Some(value) => {
                    query.push_str(r" NOT LIKE ? ESCAPE '\' COLLATE NOCASE");
                    bound_values.push(format!("{value}%"));
                }
                _ => {
                    query.push_str(r" LIKE ? ESCAPE '\' COLLATE NOCASE");
                    bound_values.push(format!("{value}%"));
                }
            }
        }
    }

    for filter in filters {
        match filter.key.as_str() {
            "all" => {
                // When querying with all we split the values into search terms
                let values: Vec<String> = filter
                    .values
                    .iter()
                    .flat_map(|value| split_search_terms(value))
                    .collect();

                // All name filter
                query.push_str("AND ((");
                write_condition_ci(query, &mut bound_values, r#""secret"."name""#, &values);

                query.push_str(") OR (");

                // All Description filter
                write_condition_ci(
                    query,
                    &mut bound_values,
                    r#""secret"."description""#,
                    &values,
                );

                // All Tag filters
                query.push_str(
                    r#") OR EXISTS (
                    SELECT 1 FROM "secrets_tags" AS "secret_tag"
                    WHERE "secret_tag"."secret_arn" = "secret"."arn"
                    AND ((
                "#,
                );

                write_condition_ci(query, &mut bound_values, r#""secret_tag"."key""#, &values);

                query.push_str(") OR (");

                write_condition_ci(query, &mut bound_values, r#""secret_tag"."value""#, &values);

                query.push_str("))))");
            }

            "name" => {
                query.push_str(" AND (");
                write_condition_cs(
                    query,
                    &mut bound_values,
                    r#""secret"."name""#,
                    &filter.values,
                );
                query.push(')');
            }

            "description" => {
                query.push_str(" AND (");
                write_condition_ci(
                    query,
                    &mut bound_values,
                    r#""secret"."description""#,
                    &filter.values,
                );
                query.push(')');
            }

            "tag-key" => {
                query.push_str(
                    r#" AND EXISTS (
                    SELECT 1 FROM "secrets_tags" AS "secret_tag"
                    WHERE "secret_tag"."secret_arn" = "secret"."arn"
                    AND (
                "#,
                );

                write_condition_cs(
                    query,
                    &mut bound_values,
                    r#""secret_tag"."key""#,
                    &filter.values,
                );
                query.push_str("))");
            }

            "tag-value" => {
                query.push_str(
                    r#" AND EXISTS (
                    SELECT 1 FROM "secrets_tags" AS "secret_tag"
                    WHERE "secret_tag"."secret_arn" = "secret"."arn"
                    AND (
                "#,
                );
                write_condition_cs(
                    query,
                    &mut bound_values,
                    r#""secret_tag"."value""#,
                    &filter.values,
                );
                query.push_str("))");
            }

            _ => {}
        }
    }

    bound_values
}

/// Get secrets filtered using the provided `filters`, will only include secrets planned for deletion
/// if `include_planned_deletions` is true.
///
/// Paginated using the provided `limit` and `offset` use `asc` to order the results by creation date
/// in ascending order, false to order descending
pub fn get_secrets_by_filter(
    db: &Connection,
    filters: &[Filter],
    include_planned_deletions: bool,
    limit: i64,
    offset: i64,
    asc: bool,
) -> DbResult<Vec<StoredSecretWithVersionStages>> {
    let mut query = r#"
        SELECT
            "secret".*,
            "secret_version"."version_id",
            "secret_version"."secret_string",
            "secret_version"."secret_binary",
            "secret_version"."created_at" AS "version_created_at",
            "secret_version"."last_accessed_at" AS "version_last_accessed_at",
            COALESCE((
                SELECT json_group_array("version_stage"."value")
                FROM "secret_version_stages" "version_stage"
                WHERE "version_stage"."secret_arn" = "secret_version"."secret_arn"
                    AND "version_stage"."version_id" = "secret_version"."version_id"
            ), '[]') AS "version_stages",
            COALESCE((
                SELECT json_group_array(
                    json_object(
                        'key', "secret_tag"."key",
                        'value', "secret_tag"."value",
                        'created_at', "secret_tag"."created_at",
                        'updated_at', "secret_tag"."updated_at"
                    )
                )
                FROM "secrets_tags" "secret_tag"
                WHERE "secret_tag"."secret_arn" = "secret"."arn"
            ), '[]') AS "version_tags",
            COALESCE((
                SELECT json_group_array(
                    json_object(
                        'version_id', "secret_version"."version_id",
                        'version_stages', COALESCE((
                            SELECT json_group_array("version_stage"."value")
                            FROM "secret_version_stages" "version_stage"
                            WHERE "version_stage"."secret_arn" = "secret_version"."secret_arn"
                                AND "version_stage"."version_id" = "secret_version"."version_id"
                        ), '[]'),
                        'created_at', "secret_version"."created_at",
                        'last_accessed_at', "secret_version"."last_accessed_at"
                    )
                )
                FROM "secrets_versions" "secret_version"
                JOIN "secret_version_stages" "version_stage"
                    ON "version_stage"."secret_arn" = "secret_version"."secret_arn"
                    AND "version_stage"."version_id" = "secret_version"."version_id"
                    AND "version_stage"."value" = 'AWSCURRENT'
                WHERE "secret_version"."secret_arn" = "secret"."arn"
            ), '[]') AS "versions"
        FROM "secrets" "secret"
        JOIN "secrets_versions" "secret_version" ON "secret_version"."secret_arn" = "secret"."arn"
        JOIN "secret_version_stages" "version_stage"
            ON "version_stage"."secret_arn" = "secret_version"."secret_arn"
            AND "version_stage"."version_id" = "secret_version"."version_id"
            AND "version_stage"."value" = 'AWSCURRENT'
        WHERE 1=1
    "#
    .to_string();

    if !include_planned_deletions {
        query.push_str(r#" AND "secret"."scheduled_delete_at" IS NULL "#);
    }

    let bound_values = push_secret_filter_where(filters, &mut query);

    // Apply ordering
    if asc {
        query.push_str(r#" ORDER BY "secret_version"."created_at" ASC "#);
    } else {
        query.push_str(r#" ORDER BY "secret_version"."created_at" DESC "#);
    }

    // Apply pagination
    query.push_str(r#"LIMIT ? OFFSET ?"#);

    db.prepare(&query)?
        .query_map(
            params_from_iter(
                bound_values
                    .iter()
                    .map(|value| value as &dyn ToSql)
                    .chain([&limit as &dyn ToSql, &offset as &dyn ToSql]),
            ),
            |row| StoredSecretWithVersionStages::try_from(row),
        )?
        .try_collect()
}

/// Get the total number of secrets filtered using the provided `filters`, will only include secrets planned for deletion
/// if `include_planned_deletions` is true.
pub fn get_secrets_count_by_filter(
    db: &Connection,
    filters: &[Filter],
    include_planned_deletions: bool,
) -> DbResult<i64> {
    let mut query = r#"
        SELECT COUNT(*)
        FROM "secrets" "secret"
        JOIN "secrets_versions" "secret_version" ON "secret_version"."secret_arn" = "secret"."arn"
        JOIN "secret_version_stages" "version_stage"
            ON "version_stage"."secret_arn" = "secret_version"."secret_arn"
            AND "version_stage"."version_id" = "secret_version"."version_id"
            AND "version_stage"."value" = 'AWSCURRENT'
        WHERE 1=1
    "#
    .to_string();

    if !include_planned_deletions {
        query.push_str(r#" AND "secret"."scheduled_delete_at" IS NULL "#);
    }

    let bound_values = push_secret_filter_where(filters, &mut query);

    db.query_one(
        &query,
        params_from_iter(bound_values.iter().map(|value| value as &dyn ToSql)),
        |row| row.get::<_, i64>(0),
    )
}

/// Get all versions of a secret
pub fn get_secret_versions(db: &Connection, secret_arn: &str) -> DbResult<Vec<SecretVersion>> {
    db.prepare(
        r#"
        SELECT
            "secret_version".*,
            COALESCE((
                SELECT json_group_array("version_stage"."value")
                FROM "secret_version_stages" "version_stage"
                WHERE "version_stage"."secret_arn" = "secret_version"."secret_arn"
                    AND "version_stage"."version_id" = "secret_version"."version_id"
            ), '[]') AS "version_stages"
        FROM "secrets_versions" "secret_version"
        WHERE "secret_version"."secret_arn" = ?
        ORDER BY "secret_version"."created_at" DESC
    "#,
    )?
    .query_map(params![secret_arn], |row| SecretVersion::try_from(row))?
    .try_collect()
}

/// Get the total number of versions for the secret
///
/// Does not include versions without at least one attached version stage
/// unless `include_deprecated` is specified
pub fn count_secret_versions(
    db: &Connection,
    secret_arn: &str,
    include_deprecated: bool,
) -> DbResult<i64> {
    db.query_one(
        r#"
           SELECT COUNT(*)
           FROM "secrets_versions" "secret_version"
           WHERE "secret_version"."secret_arn" = ? AND
               (? = TRUE OR EXISTS (
                   SELECT 1
                   FROM "secret_version_stages" "version_stage"
                   WHERE "version_stage"."secret_arn" = "secret_version"."secret_arn"
                       AND "version_stage"."version_id" = "secret_version"."version_id"
               ))
       "#,
        params![secret_arn, include_deprecated],
        |row| row.get::<_, i64>(0),
    )
}

/// Get a versions page for a secret
///
/// Does not include versions without at least one attached version stage
/// unless `include_deprecated` is specified
pub fn get_secret_versions_page(
    db: &Connection,
    secret_arn: &str,
    include_deprecated: bool,
    limit: i64,
    offset: i64,
) -> DbResult<Vec<SecretVersion>> {
    db.prepare(
        r#"
            SELECT
                "secret_version".*,
                COALESCE((
                    SELECT json_group_array("version_stage"."value")
                    FROM "secret_version_stages" "version_stage"
                    WHERE "version_stage"."secret_arn" = "secret_version"."secret_arn"
                        AND "version_stage"."version_id" = "secret_version"."version_id"
                ), '[]') AS "version_stages"
            FROM "secrets_versions" "secret_version"
            WHERE "secret_version"."secret_arn" = ? AND
                (? = TRUE OR EXISTS (
                    SELECT 1
                    FROM "secret_version_stages" "version_stage"
                    WHERE "version_stage"."secret_arn" = "secret_version"."secret_arn"
                        AND "version_stage"."version_id" = "secret_version"."version_id"
                ))
            ORDER BY "secret_version"."created_at" DESC
            LIMIT ? OFFSET ?
        "#,
    )?
    .query_map(
        params![secret_arn, include_deprecated, limit, offset],
        |row| SecretVersion::try_from(row),
    )?
    .try_collect()
}

/// Takes any secrets with over 100 versions and deletes any secrets that
/// are over 24h old until there is only 100 versions for each secret
///
/// Only allowed to delete versions that don't have a stage
pub fn delete_excess_secret_versions(db: &Connection) -> DbResult<usize> {
    let now = Utc::now();
    let cutoff = now.checked_sub_days(Days::new(1)).ok_or_else(|| {
        FromSqlError::other(Box::new(std::io::Error::other(
            "failed to create a future timestamp",
        )))
    })?;

    db.execute(
        r#"
        WITH "ranked_versions" AS (
            SELECT
                "secret_version".*,
                ROW_NUMBER() OVER (
                    PARTITION BY "secret_version"."secret_arn"
                    ORDER BY "secret_version"."created_at" DESC
                ) AS "row_number"
            FROM "secret_versions" "secret_version"
        )
        DELETE FROM "secret_versions" "secret_version"
        WHERE ("secret_arn", "version_id") IN (
            SELECT "secret_arn", "version_id"
            FROM "ranked_versions"
            WHERE "row_number" > 100
              AND "created_at" < ?
              AND NOT EXISTS (
                SELECT 1
                FROM "secret_version_stages" "version_stage"
                WHERE "version_stage"."secret_arn" = "secret_version"."secret_arn"
                    AND "version_stage"."version_id" = "secret_version"."version_id"
              )
        );
        "#,
        params![cutoff],
    )
}

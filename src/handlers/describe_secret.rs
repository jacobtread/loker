use crate::{
    database::{
        DbPool,
        secrets::{get_secret_latest_version, get_secret_versions},
    },
    handlers::{
        Handler,
        error::{AwsError, ResourceNotFoundException},
        models::{SecretId, Tag},
    },
    utils::date::datetime_to_f64,
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_DescribeSecret.html
pub struct DescribeSecretHandler;

#[derive(Deserialize, Validate)]
pub struct DescribeSecretRequest {
    #[serde(rename = "SecretId")]
    #[garde(dive)]
    secret_id: SecretId,
}

#[derive(Serialize)]
pub struct DescribeSecretResponse {
    #[serde(rename = "ARN")]
    arn: String,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "CreatedDate")]
    created_date: f64,
    #[serde(rename = "DeletedDate")]
    deleted_date: Option<f64>,
    #[serde(rename = "KmsKeyId")]
    kms_key_id: Option<String>,
    #[serde(rename = "LastAccessedDate")]
    last_accessed_date: Option<f64>,
    #[serde(rename = "LastChangedDate")]
    last_changed_date: Option<f64>,
    #[serde(rename = "LastRotatedDate")]
    last_rotated_date: Option<f64>,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "NextRotationDate")]
    next_rotation_date: Option<f64>,
    #[serde(rename = "OwningService")]
    owning_service: Option<String>,
    #[serde(rename = "PrimaryRegion")]
    primary_region: Option<String>,
    #[serde(rename = "ReplicationStatus")]
    replication_status: Option<Vec<serde_json::Value>>,
    #[serde(rename = "RotationEnabled")]
    rotation_enabled: bool,
    #[serde(rename = "RotationLambdaARN")]
    rotation_lambda_arn: Option<String>,
    #[serde(rename = "RotationRules")]
    rotation_rules: Option<serde_json::Value>,
    #[serde(rename = "Tags")]
    tags: Vec<Tag>,
    #[serde(rename = "VersionIdsToStages")]
    version_ids_to_stages: HashMap<String, Vec<String>>,
}

impl Handler for DescribeSecretHandler {
    type Request = DescribeSecretRequest;
    type Response = DescribeSecretResponse;

    #[tracing::instrument(skip_all, fields(secret_id = %request.secret_id))]
    async fn handle(db: &DbPool, request: Self::Request) -> Result<Self::Response, AwsError> {
        let SecretId(secret_id) = request.secret_id;

        let secret = get_secret_latest_version(db, &secret_id)
            .await
            //
            .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?
            //
            .ok_or(ResourceNotFoundException)?;

        let versions = get_secret_versions(db, &secret.arn)
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to get secret versions"))?;

        let most_recently_used = versions
            .iter()
            .filter_map(|version| version.last_accessed_at)
            .max();

        let tags_updated_at = secret.version_tags.iter().filter_map(|tag| tag.updated_at);

        let last_changed_date = versions
            .iter()
            .map(|version| version.created_at)
            .chain(secret.updated_at)
            .chain(tags_updated_at)
            .max();

        let version_ids_to_stages = versions
            .into_iter()
            .map(|version| (version.version_id, version.version_stages))
            .collect();

        Ok(DescribeSecretResponse {
            arn: secret.arn,
            description: secret.description,
            created_date: datetime_to_f64(secret.created_at),
            deleted_date: secret.deleted_at.map(datetime_to_f64),
            kms_key_id: None,
            last_accessed_date: most_recently_used.map(datetime_to_f64),
            last_changed_date: last_changed_date.map(datetime_to_f64),
            last_rotated_date: None,
            name: secret.name,
            next_rotation_date: None,
            owning_service: None,
            primary_region: None,
            replication_status: None,
            rotation_enabled: false,
            rotation_lambda_arn: None,
            rotation_rules: None,
            tags: secret
                .version_tags
                .into_iter()
                .map(|tag| Tag {
                    key: tag.key,
                    value: tag.value,
                })
                .collect(),
            version_ids_to_stages,
        })
    }
}

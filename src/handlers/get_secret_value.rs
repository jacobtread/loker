use crate::{
    database::{
        DbPool,
        secrets::{
            get_secret_by_version_id, get_secret_by_version_stage,
            get_secret_by_version_stage_and_id, get_secret_latest_version,
            update_secret_version_last_accessed,
        },
    },
    handlers::{
        Handler,
        error::{AwsError, InvalidRequestException, ResourceNotFoundException},
        models::{SecretId, VersionId},
    },
    utils::date::datetime_to_f64,
};
use garde::Validate;
use serde::{Deserialize, Serialize};

// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_GetSecretValue.html
pub struct GetSecretValueHandler;

#[derive(Deserialize, Validate)]
pub struct GetSecretValueRequest {
    #[serde(rename = "SecretId")]
    #[garde(dive)]
    secret_id: SecretId,

    #[serde(rename = "VersionId")]
    #[garde(dive)]
    version_id: Option<VersionId>,

    #[serde(rename = "VersionStage")]
    #[garde(inner(length(min = 1, max = 256)))]
    version_stage: Option<String>,
}

#[derive(Serialize)]
pub struct GetSecretValueResponse {
    #[serde(rename = "ARN")]
    arn: String,
    #[serde(rename = "CreatedDate")]
    created_date: f64,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "SecretString")]
    secret_string: Option<String>,
    #[serde(rename = "SecretBinary")]
    secret_binary: Option<String>,
    #[serde(rename = "VersionId")]
    version_id: String,
    #[serde(rename = "VersionStages")]
    version_stages: Vec<String>,
}

impl Handler for GetSecretValueHandler {
    type Request = GetSecretValueRequest;
    type Response = GetSecretValueResponse;

    #[tracing::instrument(skip_all, fields(secret_id = %request.secret_id))]
    async fn handle(db: &DbPool, request: Self::Request) -> Result<Self::Response, AwsError> {
        let SecretId(secret_id) = request.secret_id;
        let version_id = request.version_id.map(VersionId::into_inner);
        let version_stage = request.version_stage;

        let secret = match (&version_id, &version_stage) {
            (None, None) => get_secret_latest_version(db, &secret_id).await,
            (Some(version_id), Some(version_stage)) => {
                get_secret_by_version_stage_and_id(db, &secret_id, version_id, version_stage).await
            }
            (Some(version_id), None) => get_secret_by_version_id(db, &secret_id, version_id).await,
            (None, Some(version_stage)) => {
                get_secret_by_version_stage(db, &secret_id, version_stage).await
            }
        };

        let secret = secret
            .inspect_err(|error| tracing::error!(?error, "failed to get secret value"))?
            .ok_or(ResourceNotFoundException)?;

        // Secret is scheduled for deletion
        if secret.scheduled_delete_at.is_some() {
            return Err(InvalidRequestException.into());
        }

        // Update the access timestamp
        update_secret_version_last_accessed(db, &secret.arn, &secret.version_id)
            .await
            .inspect_err(|error| {
                tracing::error!(?error, "failed to update secret last accessed");
            })?;

        let created_at = if version_id.is_some() {
            secret.version_created_at
        } else {
            secret.created_at
        };

        Ok(GetSecretValueResponse {
            arn: secret.arn,
            created_date: datetime_to_f64(created_at),
            name: secret.name,
            secret_string: secret.secret_string,
            secret_binary: secret.secret_binary,
            version_id: secret.version_id,
            version_stages: secret.version_stages,
        })
    }
}

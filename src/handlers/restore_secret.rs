use crate::{
    database::{
        DbPool,
        secrets::{cancel_delete_secret, get_secret_latest_version},
    },
    handlers::{
        Handler,
        error::{AwsError, ResourceNotFoundException},
        models::SecretId,
    },
};
use garde::Validate;
use serde::{Deserialize, Serialize};

/// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_RestoreSecret.html
pub struct RestoreSecretHandler;

#[derive(Deserialize, Validate)]
pub struct RestoreSecretRequest {
    #[serde(rename = "SecretId")]
    #[garde(dive)]
    secret_id: SecretId,
}

#[derive(Serialize)]
pub struct RestoreSecretResponse {
    #[serde(rename = "ARN")]
    arn: String,
    #[serde(rename = "Name")]
    name: String,
}

impl Handler for RestoreSecretHandler {
    type Request = RestoreSecretRequest;
    type Response = RestoreSecretResponse;

    #[tracing::instrument(skip_all, fields(secret_id = %request.secret_id))]
    async fn handle(db: &DbPool, request: Self::Request) -> Result<Self::Response, AwsError> {
        let SecretId(secret_id) = request.secret_id;

        let secret = get_secret_latest_version(db, &secret_id)
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?
            .ok_or(ResourceNotFoundException)?;

        cancel_delete_secret(db, &secret.arn)
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?;

        Ok(RestoreSecretResponse {
            arn: secret.arn,
            name: secret.name,
        })
    }
}

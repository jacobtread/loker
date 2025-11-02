use crate::{
    database::{
        DbPool,
        secrets::{
            CreateSecretVersion, add_secret_version_stage, create_secret_version,
            get_secret_by_version_id, get_secret_latest_version, remove_secret_version_stage_any,
        },
    },
    handlers::{
        Handler,
        error::{
            AwsError, InternalServiceError, InvalidRequestException, ResourceExistsException,
            ResourceNotFoundException,
        },
        models::{ClientRequestToken, SecretBinary, SecretId, SecretString},
    },
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_PutSecretValue.html
pub struct PutSecretValueHandler;

#[derive(Deserialize, Validate)]
pub struct PutSecretValueRequest {
    #[serde(rename = "ClientRequestToken")]
    #[garde(dive)]
    client_request_token: Option<ClientRequestToken>,

    #[serde(rename = "SecretId")]
    #[garde(dive)]
    secret_id: SecretId,

    #[serde(rename = "SecretString")]
    #[garde(dive)]
    secret_string: Option<SecretString>,

    #[serde(rename = "SecretBinary")]
    #[garde(dive)]
    secret_binary: Option<SecretBinary>,

    #[serde(rename = "VersionStages")]
    #[garde(inner(length(min = 1, max = 20), inner(length(min = 1, max = 256))))]
    version_stages: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct PutSecretValueResponse {
    #[serde(rename = "ARN")]
    arn: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "VersionId")]
    version_id: String,
    #[serde(rename = "VersionStages")]
    version_stages: Vec<String>,
}

impl Handler for PutSecretValueHandler {
    type Request = PutSecretValueRequest;
    type Response = PutSecretValueResponse;

    #[tracing::instrument(skip_all, fields(secret_id = %request.secret_id))]
    async fn handle(db: &DbPool, request: Self::Request) -> Result<Self::Response, AwsError> {
        let SecretId(secret_id) = request.secret_id;
        let ClientRequestToken(version_id) = request.client_request_token.unwrap_or_default();

        let version_stages = match request.version_stages {
            Some(value) => {
                // When specifying version stages must specify at least one
                if value.is_empty() {
                    return Err(InvalidRequestException.into());
                }

                value
            }
            None => vec!["AWSCURRENT".to_string()],
        };

        let secret_string = request.secret_string.map(SecretString::into_inner);
        let secret_binary = request.secret_binary.map(SecretBinary::into_inner);

        // Must only specify one of the two
        if secret_string.is_some() && secret_binary.is_some() {
            return Err(InvalidRequestException.into());
        }

        // Must specify at least one
        if secret_string.is_none() && secret_binary.is_none() {
            return Err(InvalidRequestException.into());
        }

        let secret = get_secret_latest_version(db, &secret_id)
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?
            .ok_or(ResourceNotFoundException)?;

        let mut t = db
            .begin()
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to begin transaction"))?;

        // Create the new secret version
        if let Err(error) = create_secret_version(
            t.deref_mut(),
            CreateSecretVersion {
                secret_arn: secret.arn.clone(),
                version_id: version_id.clone(),
                secret_string: secret_string.clone(),
                secret_binary: secret_binary.clone(),
            },
        )
        .await
        {
            if let Some(error) = error.as_database_error()
                && error.is_unique_violation()
            {
                // Must rollback the transaction before attempting to use the connection
                if let Err(error) = t.rollback().await {
                    tracing::error!(?error, "failed to rollback transaction");
                }

                // Check if the secret has been created
                let secret = get_secret_by_version_id(db, &secret.arn, &version_id)
                    .await
                    .inspect_err(|error| {
                        tracing::error!(?error, "failed to determine existing version")
                    })?
                    // Unlikely but not impossible if we hit the unique violation
                    .ok_or(InternalServiceError)?;

                // If the stored version data doesn't match this is an error that
                // the resource already exists
                if secret.secret_string.ne(&secret_string)
                    || secret.secret_binary.ne(&secret_binary)
                {
                    return Err(ResourceExistsException.into());
                }

                // Another request already created this version
                return Ok(PutSecretValueResponse {
                    arn: secret.arn,
                    name: secret.name,
                    version_id: secret.version_id,
                    version_stages: secret.version_stages,
                });
            }

            tracing::error!(?error, "failed to create secret version");
            return Err(InternalServiceError.into());
        }

        // Add the requested stages
        for version_stage in &version_stages {
            remove_secret_version_stage_any(t.deref_mut(), &secret.arn, version_stage)
                .await
                .inspect_err(|error| {
                    tracing::error!(?error, "failed to remove version stage from secret")
                })?;

            // If we are re-assigning AWSCURRENT ensure that the previous secret is given AWSPREVIOUS
            if version_stage == "AWSCURRENT" {
                // Ensure nobody else has the AWSPREVIOUS stage
                remove_secret_version_stage_any(t.deref_mut(), &secret.arn, "AWSPREVIOUS")
                    .await
                    .inspect_err(|error| {
                        tracing::error!(?error, "failed to remove version stage from secret")
                    })?;

                // Add the AWSPREVIOUS stage to the old
                add_secret_version_stage(
                    t.deref_mut(),
                    &secret.arn,
                    &secret.version_id,
                    "AWSPREVIOUS",
                )
                .await
                .inspect_err(|error| {
                    tracing::error!(?error, "failed to add AWSPREVIOUS tag to secret")
                })?;
            }

            // Add the requested version stage
            add_secret_version_stage(t.deref_mut(), &secret.arn, &version_id, version_stage)
                .await
                .inspect_err(|error| tracing::error!(?error, "failed to add stage to secret"))?;
        }

        t.commit()
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to commit transaction"))?;

        Ok(PutSecretValueResponse {
            arn: secret.arn,
            name: secret.name,
            version_id,
            version_stages,
        })
    }
}

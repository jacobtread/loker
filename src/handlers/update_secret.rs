use crate::{
    database::{
        DbPool,
        secrets::{
            CreateSecretVersion, add_secret_version_stage, create_secret_version,
            get_secret_latest_version, remove_secret_version_stage,
            remove_secret_version_stage_any, update_secret_description,
        },
        transaction,
    },
    handlers::{
        Handler,
        error::{
            AwsError, InternalServiceError, InvalidRequestException, ResourceNotFoundException,
        },
        models::{ClientRequestToken, SecretBinary, SecretId, SecretString},
    },
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_UpdateSecret.html
pub struct UpdateSecretHandler;

#[derive(Deserialize, Validate)]
pub struct UpdateSecretRequest {
    #[serde(rename = "ClientRequestToken")]
    #[garde(dive)]
    client_request_token: Option<ClientRequestToken>,

    #[serde(rename = "Description")]
    #[garde(inner(length(max = 2048)))]
    description: Option<String>,

    #[serde(rename = "SecretId")]
    #[garde(dive)]
    secret_id: SecretId,

    #[serde(rename = "SecretString")]
    #[garde(dive)]
    secret_string: Option<SecretString>,

    #[serde(rename = "SecretBinary")]
    #[garde(dive)]
    secret_binary: Option<SecretBinary>,
}

#[derive(Serialize)]
pub struct UpdateSecretResponse {
    #[serde(rename = "ARN")]
    arn: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "VersionId")]
    version_id: Option<String>,
}

impl Handler for UpdateSecretHandler {
    type Request = UpdateSecretRequest;
    type Response = UpdateSecretResponse;

    #[tracing::instrument(skip_all, fields(secret_id = %request.secret_id))]
    async fn handle(db: &DbPool, request: Self::Request) -> Result<Self::Response, AwsError> {
        let UpdateSecretRequest {
            client_request_token,
            description,
            secret_id,
            secret_string,
            secret_binary,
        } = request;

        let SecretId(secret_id) = secret_id;
        let secret_string = secret_string.map(SecretString::into_inner);
        let secret_binary = secret_binary.map(SecretBinary::into_inner);

        // Must only specify one of the two
        if secret_string.is_some() && secret_binary.is_some() {
            return Err(InvalidRequestException.into());
        }

        let secret = get_secret_latest_version(db, &secret_id)
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?
            .ok_or(ResourceNotFoundException)?;

        let (secret, version_id) = transaction(db, move |t| {
            Box::pin(async move {
                if let Some(description) = description {
                    update_secret_description(t.deref_mut(), &secret.arn, &description)
                        .await
                        .inspect_err(|error| {
                            tracing::error!(?error, "failed to update secret version description")
                        })?;
                }

                let version_id = if secret_string.is_some() || secret_binary.is_some() {
                    let ClientRequestToken(version_id) = client_request_token.unwrap_or_default();

                    // Create a new current secret version
                    if let Err(error) = create_secret_version(
                        t.deref_mut(),
                        CreateSecretVersion {
                            secret_arn: secret.arn.clone(),
                            version_id: version_id.clone(),
                            secret_string,
                            secret_binary,
                        },
                    )
                    .await
                    {
                        if let Some(error) = error.as_database_error()
                            && error.is_unique_violation()
                        {
                            // Another request already created this version
                            return Ok((secret, None));
                        }

                        tracing::error!(?error, "failed to create secret version");
                        return Err(InternalServiceError.into());
                    }

                    // Remove AWSPREVIOUS from any other versions
                    remove_secret_version_stage_any(t.deref_mut(), &secret.arn, "AWSPREVIOUS")
                        .await
                        .inspect_err(|error| {
                            tracing::error!(?error, "failed to deprecate old previous secret")
                        })?;

                    // Add the AWSPREVIOUS stage to the old current
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

                    // Remove AWSCURRENT from the current version
                    remove_secret_version_stage(
                        t.deref_mut(),
                        &secret.arn,
                        &secret.version_id,
                        "AWSCURRENT",
                    )
                    .await
                    .inspect_err(|error| {
                        tracing::error!(?error, "failed to remove AWSCURRENT from old version")
                    })?;

                    // Add the AWSCURRENT stage to the new version
                    add_secret_version_stage(t.deref_mut(), &secret.arn, &version_id, "AWSCURRENT")
                        .await
                        .inspect_err(|error| {
                            tracing::error!(?error, "failed to add AWSCURRENT tag to secret")
                        })?;

                    Some(version_id)
                } else {
                    // Nothing to update
                    None
                };

                Ok::<_, AwsError>((secret, version_id))
            })
        })
        .await?;

        Ok(UpdateSecretResponse {
            arn: secret.arn,
            name: secret.name,
            version_id,
        })
    }
}

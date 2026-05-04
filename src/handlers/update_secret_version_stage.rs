use crate::{
    database::{
        DbHandle,
        ext::SqlErrorExt,
        secrets::{
            add_secret_version_stage, get_secret_latest_version, remove_secret_version_stage,
            remove_secret_version_stage_any,
        },
        transaction,
    },
    handlers::{
        Handler,
        error::{
            AwsError, InternalServiceError, InvalidRequestException, ResourceNotFoundException,
        },
        models::{SecretId, VersionId},
    },
};
use garde::Validate;
use serde::{Deserialize, Serialize};

// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_UpdateSecretVersionStage.html
pub struct UpdateSecretVersionStageHandler;

#[derive(Deserialize, Validate)]
pub struct UpdateSecretVersionStageRequest {
    #[serde(rename = "MoveToVersionId")]
    #[garde(dive)]
    move_to_version_id: Option<VersionId>,

    #[serde(rename = "RemoveFromVersionId")]
    #[garde(dive)]
    remove_from_version_id: Option<VersionId>,

    #[serde(rename = "SecretId")]
    #[garde(dive)]
    secret_id: SecretId,

    #[serde(rename = "VersionStage")]
    #[garde(length(min = 1, max = 256))]
    version_stage: String,
}

#[derive(Serialize)]
pub struct UpdateSecretVersionStageResponse {
    #[serde(rename = "ARN")]
    arn: String,
    #[serde(rename = "Name")]
    name: String,
}

impl Handler for UpdateSecretVersionStageHandler {
    type Request = UpdateSecretVersionStageRequest;
    type Response = UpdateSecretVersionStageResponse;

    #[tracing::instrument(skip_all, fields(secret_id = %request.secret_id))]
    async fn handle(db: &DbHandle, request: Self::Request) -> Result<Self::Response, AwsError> {
        let SecretId(secret_id) = request.secret_id;
        let version_stage = request.version_stage;

        let secret = db
            .call(move |db| {
                let secret = get_secret_latest_version(db, &secret_id)
                    .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?
                    .ok_or(ResourceNotFoundException)?;

                transaction(db, move |db| {
                    // Handle removing from a version
                    if let Some(VersionId(source_version_id)) = request.remove_from_version_id {
                        match remove_secret_version_stage(
                            db,
                            &secret.arn,
                            &source_version_id,
                            &version_stage,
                        ) {
                            Ok(value) => {
                                // Secret version didn't have the stage attached
                                if value < 1 {
                                    return Err(InvalidRequestException.into());
                                }
                            }
                            Err(error) => {
                                tracing::error!(?error, "failed to remove secret version stage");
                                return Err(InternalServiceError.into());
                            }
                        }
                    }

                    // If we are re-assigning AWSCURRENT ensure that the previous secret is given AWSPREVIOUS
                    if version_stage == "AWSCURRENT" && request.move_to_version_id.is_some() {
                        // Ensure nobody else has the AWSPREVIOUS stage
                        remove_secret_version_stage_any(db, &secret.arn, "AWSPREVIOUS")
                            .inspect_err(|error| {
                                tracing::error!(
                                    ?error,
                                    "failed to remove version stage from secret"
                                )
                            })?;

                        // Add the AWSPREVIOUS stage to the old current
                        add_secret_version_stage(
                            db,
                            &secret.arn,
                            &secret.version_id,
                            "AWSPREVIOUS",
                        )
                        .inspect_err(|error| {
                            tracing::error!(?error, "failed to add AWSPREVIOUS tag to secret")
                        })?;
                    }

                    if let Some(VersionId(dest_version_id)) = request.move_to_version_id
                        && let Err(error) = add_secret_version_stage(
                            db,
                            &secret.arn,
                            &dest_version_id,
                            &version_stage,
                        )
                    {
                        // Version stage is already attached to another version
                        if error.is_constraint_violation() {
                            return Err(InvalidRequestException.into());
                        }

                        tracing::error!(?error, "failed to remove secret version stage");
                        return Err(InternalServiceError.into());
                    }

                    Ok::<_, AwsError>(secret)
                })
            })
            .await?;

        Ok(UpdateSecretVersionStageResponse {
            arn: secret.arn,
            name: secret.name,
        })
    }
}

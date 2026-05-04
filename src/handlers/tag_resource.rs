use crate::{
    database::{
        DbErr, DbHandle,
        secrets::{get_secret_latest_version, put_secret_tag},
        transaction,
    },
    handlers::{
        Handler,
        error::{AwsError, ResourceNotFoundException},
        models::{SecretId, Tag},
    },
};
use garde::Validate;
use serde::{Deserialize, Serialize};

/// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_TagResource.html
pub struct TagResourceHandler;

#[derive(Deserialize, Validate)]
pub struct TagResourceRequest {
    #[serde(rename = "SecretId")]
    #[garde(dive)]
    secret_id: SecretId,

    #[serde(rename = "Tags")]
    #[garde(dive)]
    tags: Vec<Tag>,
}

#[derive(Serialize)]
pub struct TagResourceResponse {}

impl Handler for TagResourceHandler {
    type Request = TagResourceRequest;
    type Response = TagResourceResponse;

    #[tracing::instrument(skip_all, fields(secret_id = %request.secret_id))]
    async fn handle(db: &DbHandle, request: Self::Request) -> Result<Self::Response, AwsError> {
        let SecretId(secret_id) = request.secret_id;
        let tags = request.tags;

        db.call(move |db| {
            let secret = get_secret_latest_version(db, &secret_id)
                .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?
                .ok_or(ResourceNotFoundException)?;

            transaction(db, move |t| {
                // Attach all the secrets
                for tag in tags {
                    put_secret_tag(t, &secret.arn, &tag.key, &tag.value)
                        .inspect_err(|error| tracing::error!(?error, "failed to set secret tag"))?;
                }

                Ok::<_, DbErr>(())
            })?;

            Ok::<_, AwsError>(())
        })
        .await?;

        Ok(TagResourceResponse {})
    }
}

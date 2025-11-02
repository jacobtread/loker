use crate::{
    database::{
        DbPool,
        secrets::{get_secret_latest_version, put_secret_tag},
    },
    handlers::{
        Handler,
        error::{AwsError, ResourceNotFoundException},
        models::{SecretId, Tag},
    },
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

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
    async fn handle(db: &DbPool, request: Self::Request) -> Result<Self::Response, AwsError> {
        let SecretId(secret_id) = request.secret_id;
        let tags = request.tags;

        let secret = get_secret_latest_version(db, &secret_id)
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?
            .ok_or(ResourceNotFoundException)?;

        let mut t = db
            .begin()
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to begin transaction"))?;

        // Attach all the secrets
        for tag in tags {
            put_secret_tag(t.deref_mut(), &secret.arn, &tag.key, &tag.value)
                .await
                .inspect_err(|error| tracing::error!(?error, "failed to set secret tag"))?;
        }

        t.commit()
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to commit transaction"))?;

        Ok(TagResourceResponse {})
    }
}

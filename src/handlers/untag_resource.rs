use crate::{
    database::{
        DbErr, DbPool,
        secrets::{get_secret_latest_version, remove_secret_tag},
        transaction,
    },
    handlers::{
        Handler,
        error::{AwsError, ResourceNotFoundException},
        models::SecretId,
    },
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

/// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_UntagResource.html
pub struct UntagResourceHandler;

#[derive(Deserialize, Validate)]
pub struct UntagResourceRequest {
    #[serde(rename = "SecretId")]
    #[garde(dive)]
    secret_id: SecretId,

    #[serde(rename = "TagKeys")]
    #[garde(inner(length(min = 1, max = 128)))]
    tag_keys: Vec<String>,
}

#[derive(Serialize)]
pub struct UntagResourceResponse {}

impl Handler for UntagResourceHandler {
    type Request = UntagResourceRequest;
    type Response = UntagResourceResponse;

    #[tracing::instrument(skip_all, fields(secret_id = %request.secret_id))]
    async fn handle(db: &DbPool, request: Self::Request) -> Result<Self::Response, AwsError> {
        let SecretId(secret_id) = request.secret_id;
        let tag_keys = request.tag_keys;

        let secret = get_secret_latest_version(db, &secret_id)
            .await
            .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?
            .ok_or(ResourceNotFoundException)?;

        transaction(db, move |t| {
            Box::pin(async move {
                // Attach all the secrets
                for key in tag_keys {
                    remove_secret_tag(t.deref_mut(), &secret.arn, &key)
                        .await
                        .inspect_err(|error| {
                            tracing::error!(?error, "failed to remove secret tag")
                        })?;
                }

                Ok::<_, DbErr>(())
            })
        })
        .await?;

        Ok(UntagResourceResponse {})
    }
}

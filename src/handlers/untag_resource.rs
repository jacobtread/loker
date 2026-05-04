use crate::{
    database::{
        DbConnection, DbErr, DbHandle,
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
    async fn handle(db: &DbHandle, request: Self::Request) -> Result<Self::Response, AwsError> {
        let SecretId(secret_id) = request.secret_id;
        let tag_keys = request.tag_keys;

        db.call(move |db| {
            let secret = get_secret_latest_version(db, &secret_id)
                .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?
                .ok_or(ResourceNotFoundException)?;

            transaction(db, move |db| remove_secret_tags(db, &secret.arn, &tag_keys))?;

            Ok::<_, AwsError>(())
        })
        .await?;

        Ok(UntagResourceResponse {})
    }
}

fn remove_secret_tags(
    db: &DbConnection,
    secret_arn: &str,
    tag_keys: &[String],
) -> Result<(), DbErr> {
    for key in tag_keys {
        remove_secret_tag(db, secret_arn, key)
            .inspect_err(|error| tracing::error!(?error, "failed to remove secret tag"))?;
    }

    Ok(())
}

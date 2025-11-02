use crate::{
    database::{
        DbPool,
        secrets::{count_secret_versions, get_secret_latest_version, get_secret_versions_page},
    },
    handlers::{
        Handler,
        error::{AwsError, InvalidRequestException, ResourceNotFoundException},
        models::{PaginationToken, SecretId},
    },
    utils::date::datetime_to_f64,
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use tokio::join;

// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_ListSecretVersionIds.html
pub struct ListSecretVersionIdsHandler;

#[derive(Deserialize, Validate)]
pub struct ListSecretVersionIdsRequest {
    #[serde(rename = "IncludeDeprecated")]
    #[serde(default)]
    #[garde(skip)]
    include_deprecated: bool,

    #[serde(rename = "MaxResults")]
    #[serde(default = "default_max_results")]
    #[garde(range(min = 1, max = 100))]
    max_results: i32,

    #[serde(rename = "NextToken")]
    #[serde(default = "default_next_token")]
    #[garde(dive)]
    next_token: PaginationToken,

    #[serde(rename = "SecretId")]
    #[garde(dive)]
    secret_id: SecretId,
}

#[derive(Serialize)]
pub struct ListSecretVersionIdsResponse {
    #[serde(rename = "ARN")]
    arn: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "NextToken")]
    next_token: Option<String>,
    #[serde(rename = "Versions")]
    versions: Vec<SecretVersionsListEntry>,
}

#[derive(Serialize)]
pub struct SecretVersionsListEntry {
    #[serde(rename = "CreatedDate")]
    created_date: f64,
    #[serde(rename = "KmsKeyIds")]
    kms_key_ids: Option<Vec<String>>,
    #[serde(rename = "LastAccessedDate")]
    last_accessed_date: Option<f64>,
    #[serde(rename = "VersionId")]
    version_id: String,
    #[serde(rename = "VersionStages")]
    version_stages: Vec<String>,
}

fn default_max_results() -> i32 {
    100
}

fn default_next_token() -> PaginationToken {
    PaginationToken {
        page_size: 100,
        page_index: 0,
    }
}

impl Handler for ListSecretVersionIdsHandler {
    type Request = ListSecretVersionIdsRequest;
    type Response = ListSecretVersionIdsResponse;

    #[tracing::instrument(skip_all, fields(secret_id = %request.secret_id))]
    async fn handle(db: &DbPool, request: Self::Request) -> Result<Self::Response, AwsError> {
        let ListSecretVersionIdsRequest {
            include_deprecated,
            max_results,
            next_token,
            secret_id,
        } = request;

        let SecretId(secret_id) = secret_id;
        let pagination_token = next_token.page_size(max_results);

        let secret = get_secret_latest_version(db, &secret_id)
            .await
            //
            .inspect_err(|error| tracing::error!(?error, "failed to get secret"))?
            //
            .ok_or(ResourceNotFoundException)?;

        let (limit, offset) = pagination_token
            .as_query_parts()
            .ok_or(InvalidRequestException)?;

        let (versions, count) = join!(
            get_secret_versions_page(db, &secret.arn, include_deprecated, limit, offset),
            count_secret_versions(db, &secret.arn, include_deprecated),
        );

        let versions =
            versions.inspect_err(|error| tracing::error!(?error, "failed to get versions"))?;

        let count =
            count.inspect_err(|error| tracing::error!(?error, "failed to get versions count"))?;

        let next_token = pagination_token
            .get_next_page(count)
            .map(|value| value.to_string());

        let versions = versions
            .into_iter()
            .map(|version| SecretVersionsListEntry {
                created_date: datetime_to_f64(version.created_at),
                kms_key_ids: None,
                last_accessed_date: version.last_accessed_at.map(datetime_to_f64),
                version_id: version.version_id,
                version_stages: version.version_stages,
            })
            .collect();

        Ok(ListSecretVersionIdsResponse {
            arn: secret.arn,
            name: secret.name,
            next_token,
            versions,
        })
    }
}

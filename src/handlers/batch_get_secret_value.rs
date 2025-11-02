use crate::{
    database::{
        DbPool,
        secrets::{
            get_secret_latest_version, get_secrets_by_filter, get_secrets_count_by_filter,
            update_secret_version_last_accessed,
        },
    },
    handlers::{
        Handler,
        error::{AwsError, IntoErrorResponse, InvalidRequestException, ResourceNotFoundException},
        models::{APIErrorType, Filter, PaginationToken},
    },
    utils::date::datetime_to_f64,
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use tokio::join;

// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_BatchGetSecretValue.html
pub struct BatchGetSecretValueHandler;

#[derive(Deserialize, Validate)]
pub struct BatchGetSecretValueRequest {
    #[serde(rename = "Filters")]
    #[garde(dive)]
    filters: Option<Vec<Filter>>,

    #[serde(rename = "MaxResults")]
    #[garde(inner(range(min = 1, max = 20)))]
    max_results: Option<i32>,

    #[serde(rename = "NextToken")]
    #[garde(dive)]
    next_token: Option<PaginationToken>,

    #[serde(rename = "SecretIdList")]
    #[garde(inner(length(min = 1, max = 20), inner(length(min = 1, max = 2048))))]
    secret_id_list: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct BatchGetSecretValueResponse {
    #[serde(rename = "Errors")]
    errors: Vec<APIErrorType>,
    #[serde(rename = "NextToken")]
    next_token: Option<String>,
    #[serde(rename = "SecretValues")]
    secret_values: Vec<SecretValueEntry>,
}

#[derive(Serialize)]
struct SecretValueEntry {
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

fn default_max_results() -> i32 {
    20
}

fn default_next_token() -> PaginationToken {
    PaginationToken {
        page_size: 20,
        page_index: 0,
    }
}

impl Handler for BatchGetSecretValueHandler {
    type Request = BatchGetSecretValueRequest;
    type Response = BatchGetSecretValueResponse;

    #[tracing::instrument(skip_all)]
    async fn handle(db: &DbPool, request: Self::Request) -> Result<Self::Response, AwsError> {
        let mut errors: Vec<APIErrorType> = Vec::new();
        let mut secret_values: Vec<SecretValueEntry> = Vec::new();
        let mut next_token: Option<String> = None;

        match (request.filters, request.secret_id_list) {
            // Find secret values based on filters
            (Some(filters), None) => {
                let max_results = request.max_results.unwrap_or_else(default_max_results);

                let pagination_token = request
                    .next_token
                    .unwrap_or_else(default_next_token)
                    .page_size(max_results);

                let (limit, offset) = pagination_token
                    .as_query_parts()
                    .ok_or(InvalidRequestException)?;

                let (secrets, count) = join!(
                    get_secrets_by_filter(db, &filters, false, limit, offset, false),
                    get_secrets_count_by_filter(db, &filters, false),
                );

                let secrets = secrets
                    .inspect_err(|error| tracing::error!(?error, "failed to get secrets"))?;

                let count = count
                    .inspect_err(|error| tracing::error!(?error, "failed to get secrets count"))?;

                next_token = pagination_token
                    .get_next_page(count)
                    .map(|value| value.to_string());

                for secret in secrets {
                    secret_values.push(SecretValueEntry {
                        arn: secret.arn,
                        created_date: datetime_to_f64(secret.created_at),
                        name: secret.name,
                        secret_string: secret.secret_string,
                        secret_binary: secret.secret_binary,
                        version_id: secret.version_id,
                        version_stages: secret.version_stages,
                    });
                }
            }

            // Finding secrets from a list of ARNs / names
            (None, Some(secret_id_list)) => {
                for secret_id in secret_id_list {
                    let secret = get_secret_latest_version(db, &secret_id)
                        .await
                        .inspect_err(|error| {
                            tracing::error!(?error, %secret_id, "failed to load secret");
                        })?;

                    let secret = match secret {
                        Some(value) => value,
                        None => {
                            errors.push(APIErrorType {
                                error_code: Some(ResourceNotFoundException.type_name().to_string()),
                                message: Some(ResourceNotFoundException.to_string()),
                                secret_id: Some(secret_id),
                            });
                            continue;
                        }
                    };

                    update_secret_version_last_accessed(db, &secret.arn, &secret.version_id)
                        .await
                        .inspect_err(|error| {
                            tracing::error!(?error, name = %secret.name, "failed to update secret last accessed")
                        })?;

                    secret_values.push(SecretValueEntry {
                        arn: secret.arn,
                        created_date: datetime_to_f64(secret.created_at),
                        name: secret.name,
                        secret_string: secret.secret_string,
                        secret_binary: secret.secret_binary,
                        version_id: secret.version_id,
                        version_stages: secret.version_stages,
                    });
                }
            }

            // Must only specify one or the other and not both
            // and cannot pick neither
            (Some(_), Some(_)) | (None, None) => {
                return Err(InvalidRequestException.into());
            }
        }

        Ok(BatchGetSecretValueResponse {
            errors,
            next_token,
            secret_values,
        })
    }
}

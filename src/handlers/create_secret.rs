use crate::{
    database::{
        ext::SqlErrorExt,
        secrets::{
            CreateSecret, CreateSecretVersion, add_secret_version_stage, create_secret,
            create_secret_version, get_secret_by_version_id, put_secret_tag,
        },
        transaction,
    },
    handlers::{
        Handler,
        error::{AwsError, InternalServiceError, InvalidRequestException, ResourceExistsException},
        models::{ClientRequestToken, SecretBinary, SecretName, SecretString, Tag},
    },
};
use garde::Validate;
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use tokio_rusqlite::{Connection, rusqlite};

// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_CreateSecret.html
pub struct CreateSecretHandler;

#[derive(Deserialize, Validate)]
pub struct CreateSecretRequest {
    #[serde(rename = "Name")]
    #[garde(dive)]
    name: SecretName,

    #[serde(rename = "Description")]
    #[garde(inner(length(max = 2048)))]
    description: Option<String>,

    #[serde(rename = "ClientRequestToken")]
    #[garde(dive)]
    client_request_token: Option<ClientRequestToken>,

    #[serde(rename = "SecretString")]
    #[garde(dive)]
    secret_string: Option<SecretString>,

    #[serde(rename = "SecretBinary")]
    #[garde(dive)]
    secret_binary: Option<SecretBinary>,

    #[serde(rename = "Tags")]
    #[garde(dive)]
    tags: Option<Vec<Tag>>,
}

#[derive(Serialize)]
pub struct CreateSecretResponse {
    #[serde(rename = "ARN")]
    arn: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "VersionId")]
    version_id: String,
}

/// Generate a new secret ARN
///
/// Uses the mock prefix arn:aws:secretsmanager:us-east-1:1:secret:
/// and provides a randomly generated suffix as is done by the
/// official implementation
fn create_secret_arn(name: &str) -> String {
    let random_suffix: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    format!("arn:aws:secretsmanager:us-east-1:1:secret:{name}-{random_suffix}")
}

impl Handler for CreateSecretHandler {
    type Request = CreateSecretRequest;
    type Response = CreateSecretResponse;

    #[tracing::instrument(skip_all, fields(name = %request.name))]
    async fn handle(db: &Connection, request: Self::Request) -> Result<Self::Response, AwsError> {
        let SecretName(name) = request.name;
        let ClientRequestToken(version_id) = request.client_request_token.unwrap_or_default();

        let arn = create_secret_arn(&name);

        let tags = request.tags.unwrap_or_default();
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

        let response = db
            .call(move |db| {
                transaction(db, move |db| {
                    // Create the secret
                    if let CreateSecretOutcome::AlreadyFulfilled(response) =
                        create_secret_check_existing(
                            db,
                            arn.clone(),
                            name.clone(),
                            version_id.clone(),
                            request.description.clone(),
                            &secret_string,
                            &secret_binary,
                        )?
                    {
                        return Ok(response);
                    }

                    // Create the secret version
                    if let CreateSecretOutcome::AlreadyFulfilled(response) =
                        create_secret_version_check_existing(
                            db,
                            arn.clone(),
                            version_id.clone(),
                            secret_string,
                            secret_binary,
                        )?
                    {
                        return Ok(response);
                    }

                    // Attach all the tags
                    for tag in tags {
                        if let Err(error) = put_secret_tag(db, &arn, &tag.key, &tag.value) {
                            tracing::error!(?error, "failed to set secret tag");
                            return Err(InternalServiceError.into());
                        }
                    }

                    Ok::<_, AwsError>(CreateSecretResponse {
                        arn,
                        name,
                        version_id,
                    })
                })
            })
            .await?;

        Ok(response)
    }
}

enum CreateSecretOutcome {
    Success,
    AlreadyFulfilled(CreateSecretResponse),
}

/// Attempts to create a secret, if a existing secret with a matching `version_id` blocks
/// creation the duplicate request checking ensures the secret payload matches and returns
/// [CreateSecretOutcome::AlreadyFulfilled] otherwise returns a [ResourceExistsException]
fn create_secret_check_existing(
    db: &rusqlite::Connection,
    //
    arn: String,
    name: String,
    version_id: String,
    description: Option<String>,
    //
    secret_string: &Option<String>,
    secret_binary: &Option<String>,
) -> Result<CreateSecretOutcome, AwsError> {
    let create = CreateSecret {
        arn,
        name: name.clone(),
        description,
    };

    let error = match create_secret(db, create) {
        Ok(_) => return Ok(CreateSecretOutcome::Success),
        Err(error) => error,
    };

    // Only constraint violations are recoverable
    if !error.is_constraint_violation() {
        tracing::error!(?error, "failed to create secret");
        return Err(InternalServiceError.into());
    }

    // Check if the secret has been created
    let secret = get_secret_by_version_id(db, &name, &version_id)
        .inspect_err(|error| tracing::error!(?error, "failed to determine existing version"))?
        // This version we tried to store was not created so this is an already exists error
        .ok_or(ResourceExistsException)?;

    // If the stored version data doesn't match this is an error that
    // the resource already exists
    if secret.secret_string.ne(secret_string) || secret.secret_binary.ne(secret_binary) {
        return Err(ResourceExistsException.into());
    }

    // Request has already been fulfilled
    Ok(CreateSecretOutcome::AlreadyFulfilled(
        CreateSecretResponse {
            arn: secret.arn,
            name,
            version_id,
        },
    ))
}

/// Attempts to create a secret, if a existing secret version with a matching `version_id` blocks
/// creation the duplicate request checking ensures the secret payload matches and returns
/// [CreateSecretOutcome::AlreadyFulfilled] otherwise returns a [ResourceExistsException]
fn create_secret_version_check_existing(
    db: &rusqlite::Connection,
    //
    arn: String,
    version_id: String,
    //
    secret_string: Option<String>,
    secret_binary: Option<String>,
) -> Result<CreateSecretOutcome, AwsError> {
    // Create the initial secret version
    if let Err(error) = create_secret_version(
        db,
        CreateSecretVersion {
            secret_arn: arn.clone(),
            version_id: version_id.clone(),
            secret_string: secret_string.clone(),
            secret_binary: secret_binary.clone(),
        },
    ) {
        // Only constraint violations are recoverable
        if !error.is_constraint_violation() {
            tracing::error!(?error, "failed to create secret version");
            return Err(InternalServiceError.into());
        }

        // Check if the secret has been created
        let secret = get_secret_by_version_id(db, &arn, &version_id)
            .map_err(|error| {
                tracing::error!(?error, "failed to determine existing version");
                InternalServiceError
            })?
            // Shouldn't be possible if we hit the unique violation
            .ok_or(InternalServiceError)?;

        // If the stored version data doesn't match this is an error that
        // the resource already exists
        if secret.secret_string.ne(&secret_string) || secret.secret_binary.ne(&secret_binary) {
            return Err(ResourceExistsException.into());
        }

        // Request has already been fulfilled
        return Ok(CreateSecretOutcome::AlreadyFulfilled(
            CreateSecretResponse {
                arn: secret.arn,
                name: secret.name,
                version_id: secret.version_id,
            },
        ));
    }

    // Add the AWSCURRENT stage to the new version
    if let Err(error) = add_secret_version_stage(db, &arn, &version_id, "AWSCURRENT") {
        tracing::error!(?error, "failed to add AWSPREVIOUS tag to secret");
        return Err(InternalServiceError.into());
    }

    Ok(CreateSecretOutcome::Success)
}

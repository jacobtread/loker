use aws_sdk_secretsmanager::{
    error::SdkError,
    operation::put_secret_value::PutSecretValueError,
    primitives::Blob,
    types::{
        Tag,
        error::{InvalidRequestException, ResourceExistsException},
    },
};
use loker::database::secrets::get_secret_versions;
use uuid::Uuid;

use crate::common::test_server;

mod common;

/// Tests that a string secret can be updated to a new value
#[tokio::test]
async fn test_put_secret_value_string_secret_success() {
    let (client, _server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let get_response = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Created ARN should match
    assert_eq!(get_response.arn(), create_response.arn());

    // Retrieved value should match created
    assert_eq!(get_response.secret_string(), Some("test"));

    let put_response = client
        .put_secret_value()
        .secret_id("test")
        .secret_string("test-updated")
        .send()
        .await
        .unwrap();

    // ARN should match
    assert_eq!(put_response.arn(), create_response.arn());

    // Name should match
    assert_eq!(put_response.name(), create_response.name());

    // Version number should have changed
    assert_ne!(put_response.version_id(), create_response.version_id());

    // When no stage is present the stage matched should be
    assert_eq!(put_response.version_stages(), &["AWSCURRENT".to_string()]);

    let get_response = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // ARN should still match
    assert_eq!(get_response.arn(), create_response.arn());

    // Retrieved value should match created
    assert_eq!(get_response.secret_string(), Some("test-updated"));

    // Version number should have changed
    assert_eq!(get_response.version_id(), put_response.version_id());

    // Should be in the current stage
    assert_eq!(get_response.version_stages(), &["AWSCURRENT".to_string()]);
}

/// Tests that a binary secret can be updated to a new value
#[tokio::test]
async fn test_put_secret_value_binary_secret_success() {
    let (client, _server) = test_server().await;

    let binary_secret = Blob::new(b"TEST");

    let create_response = client
        .create_secret()
        .name("test")
        .secret_binary(binary_secret.clone())
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let get_response = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Created ARN should match
    assert_eq!(get_response.arn(), create_response.arn());

    // Retrieved value should match created
    assert_eq!(get_response.secret_binary(), Some(&binary_secret));

    let binary_secret = Blob::new(b"TEST2");

    let put_response = client
        .put_secret_value()
        .secret_id("test")
        .secret_binary(binary_secret.clone())
        .send()
        .await
        .unwrap();

    // ARN should match
    assert_eq!(put_response.arn(), create_response.arn());

    // Name should match
    assert_eq!(put_response.name(), create_response.name());

    // Version number should have changed
    assert_ne!(put_response.version_id(), create_response.version_id());

    // When no stage is present the stage matched should be
    assert_eq!(put_response.version_stages(), &["AWSCURRENT".to_string()]);

    let get_response = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // ARN should still match
    assert_eq!(get_response.arn(), create_response.arn());

    // Retrieved value should match created
    assert_eq!(get_response.secret_binary(), Some(&binary_secret));

    // Version number should have changed
    assert_eq!(get_response.version_id(), put_response.version_id());

    // Should be in the current stage
    assert_eq!(get_response.version_stages(), &["AWSCURRENT".to_string()]);
}

/// Tests that not specifying a secret value will error
#[tokio::test]
async fn test_put_secret_value_missing_value_error() {
    let (client, _server) = test_server().await;

    let _create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let put_error = client
        .put_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap_err();

    let put_error = match put_error {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: InvalidRequestException = match put_error.into_err() {
        PutSecretValueError::InvalidRequestException(error) => error,
        error => panic!("expected PutSecretValueError::InvalidRequestException got {error:?}"),
    };
}

/// Tests that specifying both a string and binary secret value should
/// error, only one of the two should be able to be provided
#[tokio::test]
async fn test_put_secret_value_both_secret_type_error() {
    let (client, _server) = test_server().await;

    let _create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let binary_secret = Blob::new(b"TEST");

    let put_error = client
        .put_secret_value()
        .secret_id("test")
        .secret_string("test-updated")
        .secret_binary(binary_secret)
        .send()
        .await
        .unwrap_err();

    let put_error = match put_error {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: InvalidRequestException = match put_error.into_err() {
        PutSecretValueError::InvalidRequestException(error) => error,
        error => panic!("expected PutSecretValueError::InvalidRequestException got {error:?}"),
    };
}

/// Tests that secret stages are updated correctly between PutSecretValue calls
#[tokio::test]
async fn test_put_secret_value_version_stage() {
    let (client, server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let version_2 = client
        .put_secret_value()
        .secret_id("test")
        .secret_string("test-2")
        .send()
        .await
        .unwrap();

    let version_3 = client
        .put_secret_value()
        .secret_id("test")
        .secret_string("test-3")
        .send()
        .await
        .unwrap();

    let arn = create_response.arn().unwrap().to_string();

    let versions = server
        .db
        .call_unwrap(move |db| get_secret_versions(db, &arn))
        .await
        .unwrap();

    let initial_version = versions
        .iter()
        .find(|version| version.version_id == create_response.version_id().unwrap());
    let version_2 = versions
        .iter()
        .find(|version| version.version_id == version_2.version_id().unwrap());
    let version_3 = versions
        .iter()
        .find(|version| version.version_id == version_3.version_id().unwrap());

    // Initial version should have no version stages
    assert!(initial_version.unwrap().version_stages.is_empty());
    assert_eq!(
        version_2.unwrap().version_stages,
        vec!["AWSPREVIOUS".to_string()]
    );
    assert_eq!(
        version_3.unwrap().version_stages,
        vec!["AWSCURRENT".to_string()]
    );
}

/// Tests that trying to make the same requesting using a ClientRequestToken for a already
/// successful version will silently succeed with the existing secret details
/// rather than error as long as the value matches
#[tokio::test]
async fn test_put_secret_value_retry_token() {
    let (client, _server) = test_server().await;

    let client_request_token = Uuid::new_v4().to_string();

    let _create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let put_response_1 = client
        .put_secret_value()
        .secret_id("test")
        .secret_string("test-updated")
        .client_request_token(&client_request_token)
        .send()
        .await
        .unwrap();

    let put_response_2 = client
        .put_secret_value()
        .secret_id("test")
        .secret_string("test-updated")
        .client_request_token(&client_request_token)
        .send()
        .await
        .unwrap();

    assert_eq!(put_response_1.arn(), put_response_2.arn());
    assert_eq!(put_response_1.name(), put_response_2.name());
    assert_eq!(put_response_1.version_id(), put_response_2.version_id());
    assert_eq!(
        put_response_1.version_stages(),
        put_response_2.version_stages()
    );
}

/// Tests that trying to make a second request using the same ClientRequestToken after the
/// first succeeded but using a different value should error
#[tokio::test]
async fn test_put_secret_value_retry_token_different_error() {
    let (client, _server) = test_server().await;

    let client_request_token = Uuid::new_v4().to_string();

    let _create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let _put_response_1 = client
        .put_secret_value()
        .secret_id("test")
        .secret_string("test-updated")
        .client_request_token(&client_request_token)
        .send()
        .await
        .unwrap();

    let put_response_err = client
        .put_secret_value()
        .secret_id("test")
        .secret_string("test-updated-different")
        .client_request_token(&client_request_token)
        .send()
        .await
        .unwrap_err();

    let list_error = match put_response_err {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: ResourceExistsException = match list_error.into_err() {
        PutSecretValueError::ResourceExistsException(error) => error,
        error => panic!("expected PutSecretValueError::ResourceExistsException got {error:?}"),
    };
}

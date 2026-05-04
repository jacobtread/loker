use aws_sdk_secretsmanager::{
    error::SdkError,
    operation::{delete_secret::DeleteSecretError, get_secret_value::GetSecretValueError},
    types::{
        Tag,
        error::{InvalidRequestException, ResourceNotFoundException},
    },
};
use chrono::{Days, Utc};
use loker::database::secrets::{delete_scheduled_secrets, get_scheduled_secret_deletions};

use crate::common::test_server;

mod common;

/// Tests that requesting the immediate deletion of a secret succeeds
#[tokio::test]
async fn test_delete_secret_immediate_success() {
    let (client, _server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let delete_response = client
        .delete_secret()
        .secret_id("test")
        .force_delete_without_recovery(true)
        .send()
        .await
        .unwrap();

    // ARN and name should match
    assert_eq!(delete_response.arn(), create_response.arn());
    assert_eq!(delete_response.name(), create_response.name());

    // Deletion date should be present and non zero
    assert!(delete_response.deletion_date().unwrap().secs() > 0);

    let get_error = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap_err();

    let get_error = match get_error {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: ResourceNotFoundException = match get_error.into_err() {
        GetSecretValueError::ResourceNotFoundException(error) => error,
        error => panic!("expected GetSecretValueError::ResourceNotFoundException got {error:?}"),
    };
}

/// Tests that requesting a scheduled deletion succeeds
#[tokio::test]
async fn test_delete_secret_scheduled_success() {
    let (client, server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let delete_response = client
        .delete_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // ARN and name should match
    assert_eq!(delete_response.arn(), create_response.arn());
    assert_eq!(delete_response.name(), create_response.name());

    // Scheduled deletions should include the ARN
    let deletions = server
        .db
        .call_unwrap(|db| get_scheduled_secret_deletions(db))
        .await
        .unwrap();
    let arn = deletions.first().expect("expecting deletion arn");
    assert_eq!(arn, delete_response.arn().unwrap());

    // Deletion date should be present and non zero
    assert!(delete_response.deletion_date().unwrap().secs() > 0);

    // Attempting to load the secret should give a InvalidRequestException
    let get_error = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap_err();

    let get_error = match get_error {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: InvalidRequestException = match get_error.into_err() {
        GetSecretValueError::InvalidRequestException(error) => error,
        error => panic!("expected GetSecretValueError::InvalidRequestException got {error:?}"),
    };

    // Run the scheduled deletion logic
    let now = Utc::now()
        // Add enough days to be past the expiry
        .checked_add_days(Days::new(31))
        .unwrap();
    server
        .db
        .call_unwrap(move |db| delete_scheduled_secrets(db, now))
        .await
        .unwrap();

    // Attempting to load the secret should give a ResourceNotFoundException
    let get_error = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap_err();

    let get_error = match get_error {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: ResourceNotFoundException = match get_error.into_err() {
        GetSecretValueError::ResourceNotFoundException(error) => error,
        error => panic!("expected GetSecretValueError::ResourceNotFoundException got {error:?}"),
    };
}

/// Tests that requesting a scheduled deletion succeeds even if it was already
/// scheduled for deletion, in this case the same timestamp from the original
/// delete response is returned again
#[tokio::test]
async fn test_delete_secret_scheduled_twice_success() {
    let (client, _server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let delete_response_1 = client
        .delete_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // ARN and name should match
    assert_eq!(delete_response_1.arn(), create_response.arn());
    assert_eq!(delete_response_1.name(), create_response.name());

    let delete_response_2 = client
        .delete_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // ARN and name should match
    assert_eq!(delete_response_2.arn(), delete_response_1.arn());
    assert_eq!(delete_response_2.name(), delete_response_1.name());
    assert_eq!(
        delete_response_2.deletion_date(),
        delete_response_1.deletion_date()
    );
}

/// Tests that performing a forced delete twice will fail
#[tokio::test]
async fn test_delete_secret_immediate_twice_error() {
    let (client, _server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let delete_response = client
        .delete_secret()
        .secret_id("test")
        .force_delete_without_recovery(true)
        .send()
        .await
        .unwrap();

    // ARN and name should match
    assert_eq!(delete_response.arn(), create_response.arn());
    assert_eq!(delete_response.name(), create_response.name());

    let delete_error = client
        .delete_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap_err();

    let delete_error = match delete_error {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: ResourceNotFoundException = match delete_error.into_err() {
        DeleteSecretError::ResourceNotFoundException(error) => error,
        error => panic!("expected DeleteSecretError::ResourceNotFoundException got {error:?}"),
    };
}

/// Tests that performing a forced delete on an unknown secret errors
#[tokio::test]
async fn test_delete_secret_immediate_unknown_error() {
    let (client, _server) = test_server().await;

    let delete_error = client
        .delete_secret()
        .secret_id("test")
        .force_delete_without_recovery(true)
        .send()
        .await
        .unwrap_err();

    let delete_error = match delete_error {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: ResourceNotFoundException = match delete_error.into_err() {
        DeleteSecretError::ResourceNotFoundException(error) => error,
        error => panic!("expected DeleteSecretError::ResourceNotFoundException got {error:?}"),
    };
}

/// Tests that performing a scheduled delete on an unknown secret errors
#[tokio::test]
async fn test_delete_secret_scheduled_unknown_error() {
    let (client, _server) = test_server().await;

    let delete_error = client
        .delete_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap_err();

    let delete_error = match delete_error {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: ResourceNotFoundException = match delete_error.into_err() {
        DeleteSecretError::ResourceNotFoundException(error) => error,
        error => panic!("expected DeleteSecretError::ResourceNotFoundException got {error:?}"),
    };
}

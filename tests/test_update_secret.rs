use crate::common::test_server;
use aws_sdk_secretsmanager::{
    error::SdkError,
    operation::update_secret::UpdateSecretError,
    primitives::Blob,
    types::{Tag, error::InvalidRequestException},
};
use loker::database::secrets::get_secret_versions;

mod common;

/// Tests that the description of a secret can be set
#[tokio::test]
async fn test_update_secret_set_description_success() {
    let (client, _server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let describe_response = client
        .describe_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Should have no description
    assert_eq!(describe_response.description(), None);

    let update_response = client
        .update_secret()
        .secret_id("test")
        .description("this is the secret description")
        .send()
        .await
        .unwrap();

    // ARN and name should match create
    assert_eq!(update_response.arn(), create_response.arn());
    assert_eq!(update_response.name(), create_response.name());

    // Version ID should not be present as the value was not changed
    assert_eq!(update_response.version_id(), None);

    let describe_response = client
        .describe_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Should have the new description
    assert_eq!(
        describe_response.description(),
        Some("this is the secret description")
    );
}

/// Tests that the description of a secret can be updated
#[tokio::test]
async fn test_update_secret_update_description_success() {
    let (client, _server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .description("original description")
        .tags(Tag::builder().key("test-tag").value("test-value").build())
        .send()
        .await
        .unwrap();

    let describe_response = client
        .describe_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Should have no description
    assert_eq!(
        describe_response.description(),
        Some("original description")
    );

    let update_response = client
        .update_secret()
        .secret_id("test")
        .description("this is the secret description")
        .send()
        .await
        .unwrap();

    // ARN and name should match create
    assert_eq!(update_response.arn(), create_response.arn());
    assert_eq!(update_response.name(), create_response.name());

    // Version ID should not be present as the value was not changed
    assert_eq!(update_response.version_id(), None);

    let describe_response = client
        .describe_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Should have the new description
    assert_eq!(
        describe_response.description(),
        Some("this is the secret description")
    );
}

/// Tests that both the secret value and description can be updated (When description is already set)
#[tokio::test]
async fn test_update_secret_update_secret_string_with_update_description_success() {
    let (client, server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .description("original description")
        .secret_string("test")
        .send()
        .await
        .unwrap();

    let update_response = client
        .update_secret()
        .secret_id("test")
        .secret_string("test-2")
        .description("this is the secret description")
        .send()
        .await
        .unwrap();

    let arn = create_response.arn().unwrap().to_string();
    let versions = server
        .db
        .call_unwrap(move |db| get_secret_versions(db, &arn))
        .await
        .unwrap();

    // Should have 2 versions after the change
    assert_eq!(versions.len(), 2);

    // ARN and name should match create
    assert_eq!(update_response.arn(), create_response.arn());
    assert_eq!(update_response.name(), create_response.name());

    // Version ID should be present as a new version should have been created
    assert!(update_response.version_id().is_some());

    let get_response = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Get should respond with the new version
    assert_eq!(get_response.version_id(), update_response.version_id());

    // Should have the new description
    assert_eq!(get_response.secret_string(), Some("test-2"));

    let describe_response = client
        .describe_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Should have the new description
    assert_eq!(
        describe_response.description(),
        Some("this is the secret description")
    );
}

/// Tests that both the secret value and description can be updated (When description is not set)
#[tokio::test]
async fn test_update_secret_update_secret_string_with_set_description_success() {
    let (client, server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .send()
        .await
        .unwrap();

    let update_response = client
        .update_secret()
        .secret_id("test")
        .secret_string("test-2")
        .description("this is the secret description")
        .send()
        .await
        .unwrap();

    let arn = create_response.arn().unwrap().to_string();
    let versions = server
        .db
        .call_unwrap(move |db| get_secret_versions(db, &arn))
        .await
        .unwrap();

    // Should have 2 versions after the change
    assert_eq!(versions.len(), 2);

    // ARN and name should match create
    assert_eq!(update_response.arn(), create_response.arn());
    assert_eq!(update_response.name(), create_response.name());

    // Version ID should be present as a new version should have been created
    assert!(update_response.version_id().is_some());

    let get_response = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Get should respond with the new version
    assert_eq!(get_response.version_id(), update_response.version_id());

    // Should have the new description
    assert_eq!(get_response.secret_string(), Some("test-2"));

    let describe_response = client
        .describe_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Should have the new description
    assert_eq!(
        describe_response.description(),
        Some("this is the secret description")
    );
}

/// Tests that the secret value can be updated to a different string value
#[tokio::test]
async fn test_update_secret_update_secret_string_success() {
    let (client, server) = test_server().await;

    let create_response = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .send()
        .await
        .unwrap();

    let update_response = client
        .update_secret()
        .secret_id("test")
        .secret_string("test-2")
        .send()
        .await
        .unwrap();

    let arn = create_response.arn().unwrap().to_string();
    let versions = server
        .db
        .call_unwrap(move |db| get_secret_versions(db, &arn))
        .await
        .unwrap();

    // Should have 2 versions after the change
    assert_eq!(versions.len(), 2);

    // ARN and name should match create
    assert_eq!(update_response.arn(), create_response.arn());
    assert_eq!(update_response.name(), create_response.name());

    // Version ID should be present as a new version should have been created
    assert!(update_response.version_id().is_some());

    let get_response = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Get should respond with the new version
    assert_eq!(get_response.version_id(), update_response.version_id());

    // Should have the new description
    assert_eq!(get_response.secret_string(), Some("test-2"));
}

/// Tests that the secret value can be updated to a different binary value
#[tokio::test]
async fn test_update_secret_update_secret_binary_success() {
    let (client, server) = test_server().await;

    let secret = Blob::new(b"secret");
    let secret_2 = Blob::new(b"secret-2");

    let create_response = client
        .create_secret()
        .name("test")
        .secret_binary(secret.clone())
        .send()
        .await
        .unwrap();

    let update_response = client
        .update_secret()
        .secret_id("test")
        .secret_binary(secret_2.clone())
        .send()
        .await
        .unwrap();

    let arn = create_response.arn().unwrap().to_string();
    let versions = server
        .db
        .call_unwrap(move |db| get_secret_versions(db, &arn))
        .await
        .unwrap();

    // Should have 2 versions after the change
    assert_eq!(versions.len(), 2);

    // ARN and name should match create
    assert_eq!(update_response.arn(), create_response.arn());
    assert_eq!(update_response.name(), create_response.name());

    // Version ID should be present as a new version should have been created
    assert!(update_response.version_id().is_some());

    let get_response = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Get should respond with the new version
    assert_eq!(get_response.version_id(), update_response.version_id());

    // Should have the new description
    assert_eq!(get_response.secret_binary(), Some(&secret_2));
}

/// Tests that both the secret value and description can be updated (When description is already set)
#[tokio::test]
async fn test_update_secret_update_secret_binary_success_with_update_description_success() {
    let (client, server) = test_server().await;

    let secret = Blob::new(b"secret");
    let secret_2 = Blob::new(b"secret-2");

    let create_response = client
        .create_secret()
        .name("test")
        .description("original description")
        .secret_binary(secret.clone())
        .send()
        .await
        .unwrap();

    let update_response = client
        .update_secret()
        .secret_id("test")
        .secret_binary(secret_2.clone())
        .description("this is the secret description")
        .send()
        .await
        .unwrap();

    let arn = create_response.arn().unwrap().to_string();
    let versions = server
        .db
        .call_unwrap(move |db| get_secret_versions(db, &arn))
        .await
        .unwrap();

    // Should have 2 versions after the change
    assert_eq!(versions.len(), 2);

    // ARN and name should match create
    assert_eq!(update_response.arn(), create_response.arn());
    assert_eq!(update_response.name(), create_response.name());

    // Version ID should be present as a new version should have been created
    assert!(update_response.version_id().is_some());

    let get_response = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Get should respond with the new version
    assert_eq!(get_response.version_id(), update_response.version_id());

    // Should have the new description
    assert_eq!(get_response.secret_binary(), Some(&secret_2));

    let describe_response = client
        .describe_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Should have the new description
    assert_eq!(
        describe_response.description(),
        Some("this is the secret description")
    );
}

/// Tests that both the secret value and description can be updated (When description is not set)
#[tokio::test]
async fn test_update_secret_update_secret_binary_success_with_set_description_success() {
    let (client, server) = test_server().await;

    let secret = Blob::new(b"secret");
    let secret_2 = Blob::new(b"secret-2");

    let create_response = client
        .create_secret()
        .name("test")
        .secret_binary(secret.clone())
        .send()
        .await
        .unwrap();

    let update_response = client
        .update_secret()
        .secret_id("test")
        .secret_binary(secret_2.clone())
        .description("this is the secret description")
        .send()
        .await
        .unwrap();

    let arn = create_response.arn().unwrap().to_string();
    let versions = server
        .db
        .call_unwrap(move |db| get_secret_versions(db, &arn))
        .await
        .unwrap();

    // Should have 2 versions after the change
    assert_eq!(versions.len(), 2);

    // ARN and name should match create
    assert_eq!(update_response.arn(), create_response.arn());
    assert_eq!(update_response.name(), create_response.name());

    // Version ID should be present as a new version should have been created
    assert!(update_response.version_id().is_some());

    let get_response = client
        .get_secret_value()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Get should respond with the new version
    assert_eq!(get_response.version_id(), update_response.version_id());

    // Should have the new description
    assert_eq!(get_response.secret_binary(), Some(&secret_2));

    let describe_response = client
        .describe_secret()
        .secret_id("test")
        .send()
        .await
        .unwrap();

    // Should have the new description
    assert_eq!(
        describe_response.description(),
        Some("this is the secret description")
    );
}

/// Tests that trying to set the secret value to both a binary and a string
/// should fail
#[tokio::test]
async fn test_update_secret_update_both_error() {
    let (client, _server) = test_server().await;

    let secret = Blob::new(b"secret");
    let secret_2 = Blob::new(b"secret-2");

    let _create_response = client
        .create_secret()
        .name("test")
        .secret_binary(secret.clone())
        .send()
        .await
        .unwrap();

    let update_err = client
        .update_secret()
        .secret_id("test")
        .secret_string("test-2")
        .secret_binary(secret_2.clone())
        .send()
        .await
        .unwrap_err();

    let update_err = match update_err {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: InvalidRequestException = match update_err.into_err() {
        UpdateSecretError::InvalidRequestException(error) => error,
        error => panic!("expected UpdateSecretError::InvalidRequestException got {error:?}"),
    };
}

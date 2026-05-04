use aws_sdk_secretsmanager::{
    error::SdkError,
    operation::{
        batch_get_secret_value::BatchGetSecretValueError, create_secret::CreateSecretOutput,
    },
    types::{Filter, Tag, error::InvalidRequestException},
};

use crate::common::test_server;

mod common;

/// Tests that BatchGetSecretValue finds all the expected secrets
/// when searching by name
#[tokio::test]
async fn test_batch_get_secret_value_find_by_secret_names() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .send()
        .await
        .unwrap();

    let _create_response_4 = client
        .create_secret()
        .name("test-4")
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .secret_id_list("test-1")
        .secret_id_list("test-2")
        .secret_id_list("test-3")
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_1.arn());
    assert_eq!(secret_1.name(), create_response_1.name());
    assert_eq!(secret_1.version_id(), create_response_1.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-1"));

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());
    assert_eq!(secret_2.name(), create_response_2.name());
    assert_eq!(secret_2.version_id(), create_response_2.version_id());
    assert_eq!(secret_2.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_2.secret_binary(), None);
    assert_eq!(secret_2.secret_string(), Some("test-2"));

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_3.arn());
    assert_eq!(secret_3.name(), create_response_3.name());
    assert_eq!(secret_3.version_id(), create_response_3.version_id());
    assert_eq!(secret_3.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_3.secret_binary(), None);
    assert_eq!(secret_3.secret_string(), Some("test-3"));
}

/// Tests that BatchGetSecretValue finds all the expected secrets
/// when searching by ARN
#[tokio::test]
async fn test_batch_get_secret_value_find_by_secret_arn() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .send()
        .await
        .unwrap();

    let _create_response_4 = client
        .create_secret()
        .name("test-4")
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .secret_id_list(create_response_1.arn().unwrap())
        .secret_id_list(create_response_2.arn().unwrap())
        .secret_id_list(create_response_3.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_1.arn());
    assert_eq!(secret_1.name(), create_response_1.name());
    assert_eq!(secret_1.version_id(), create_response_1.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-1"));

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());
    assert_eq!(secret_2.name(), create_response_2.name());
    assert_eq!(secret_2.version_id(), create_response_2.version_id());
    assert_eq!(secret_2.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_2.secret_binary(), None);
    assert_eq!(secret_2.secret_string(), Some("test-2"));

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_3.arn());
    assert_eq!(secret_3.name(), create_response_3.name());
    assert_eq!(secret_3.version_id(), create_response_3.version_id());
    assert_eq!(secret_3.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_3.secret_binary(), None);
    assert_eq!(secret_3.secret_string(), Some("test-3"));
}

/// Tests that BatchGetSecretValue finds all the expected secrets
/// when searching by name or ARN in the same query
#[tokio::test]
async fn test_batch_get_secret_value_find_by_mixed() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .send()
        .await
        .unwrap();

    let _create_response_4 = client
        .create_secret()
        .name("test-4")
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .secret_id_list(create_response_1.arn().unwrap())
        .secret_id_list("test-2")
        .secret_id_list(create_response_3.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_1.arn());
    assert_eq!(secret_1.name(), create_response_1.name());
    assert_eq!(secret_1.version_id(), create_response_1.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-1"));

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());
    assert_eq!(secret_2.name(), create_response_2.name());
    assert_eq!(secret_2.version_id(), create_response_2.version_id());
    assert_eq!(secret_2.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_2.secret_binary(), None);
    assert_eq!(secret_2.secret_string(), Some("test-2"));

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_3.arn());
    assert_eq!(secret_3.name(), create_response_3.name());
    assert_eq!(secret_3.version_id(), create_response_3.version_id());
    assert_eq!(secret_3.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_3.secret_binary(), None);
    assert_eq!(secret_3.secret_string(), Some("test-3"));
}

/// Tests that BatchGetSecretValue finds all the expected secrets
/// when searching using a filter for name
#[tokio::test]
async fn test_batch_get_secret_value_find_by_filter_name() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .send()
        .await
        .unwrap();

    let _create_response_4 = client
        .create_secret()
        .name("should-not-match-test-4")
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let _create_response_5 = client
        .create_secret()
        .name("TEST-4")
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .filters(
            Filter::builder()
                .key(aws_sdk_secretsmanager::types::FilterNameStringType::Name)
                .values("test-")
                .build(),
        )
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_3.arn());
    assert_eq!(secret_1.name(), create_response_3.name());
    assert_eq!(secret_1.version_id(), create_response_3.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-3"));

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());
    assert_eq!(secret_2.name(), create_response_2.name());
    assert_eq!(secret_2.version_id(), create_response_2.version_id());
    assert_eq!(secret_2.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_2.secret_binary(), None);
    assert_eq!(secret_2.secret_string(), Some("test-2"));

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_1.arn());
    assert_eq!(secret_3.name(), create_response_1.name());
    assert_eq!(secret_3.version_id(), create_response_1.version_id());
    assert_eq!(secret_3.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_3.secret_binary(), None);
    assert_eq!(secret_3.secret_string(), Some("test-1"));
}

/// Tests that BatchGetSecretValue will update the last accessed timestamp of
/// all the retrieved secrets when using a filter
#[tokio::test]
async fn test_batch_get_secret_value_find_by_filter_name_updates_last_accessed() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .send()
        .await
        .unwrap();

    let create_response_4 = client
        .create_secret()
        .name("should-not-match-test-4")
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let create_response_5 = client
        .create_secret()
        .name("TEST-4")
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .filters(
            Filter::builder()
                .key(aws_sdk_secretsmanager::types::FilterNameStringType::Name)
                .values("test-")
                .build(),
        )
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_3.arn());

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_1.arn());

    let secret_1_describe = client
        .describe_secret()
        .secret_id(secret_1.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_1_describe.last_accessed_date().is_some());

    let secret_1_describe = client
        .describe_secret()
        .secret_id(secret_1.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_1_describe.last_accessed_date().is_some());

    let secret_2_describe = client
        .describe_secret()
        .secret_id(secret_2.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_2_describe.last_accessed_date().is_some());

    let secret_3_describe = client
        .describe_secret()
        .secret_id(secret_3.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_3_describe.last_accessed_date().is_some());

    let secret_4_describe = client
        .describe_secret()
        .secret_id(create_response_4.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_4_describe.last_accessed_date().is_none());

    let secret_5_describe = client
        .describe_secret()
        .secret_id(create_response_5.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_5_describe.last_accessed_date().is_none());
}

/// Tests that BatchGetSecretValue finds all the expected secrets
/// when searching by name and updates the last accessed timestamp
#[tokio::test]
async fn test_batch_get_secret_value_find_by_secret_names_updates_last_accessed() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .send()
        .await
        .unwrap();

    let create_response_4 = client
        .create_secret()
        .name("test-4")
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let create_response_5 = client
        .create_secret()
        .name("test-5")
        .secret_string("test-5")
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .secret_id_list("test-1")
        .secret_id_list("test-2")
        .secret_id_list("test-3")
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_1.arn());

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_3.arn());

    let secret_1_describe = client
        .describe_secret()
        .secret_id(secret_1.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_1_describe.last_accessed_date().is_some());

    let secret_1_describe = client
        .describe_secret()
        .secret_id(secret_1.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_1_describe.last_accessed_date().is_some());

    let secret_2_describe = client
        .describe_secret()
        .secret_id(secret_2.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_2_describe.last_accessed_date().is_some());

    let secret_3_describe = client
        .describe_secret()
        .secret_id(secret_3.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_3_describe.last_accessed_date().is_some());

    let secret_4_describe = client
        .describe_secret()
        .secret_id(create_response_4.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_4_describe.last_accessed_date().is_none());

    let secret_5_describe = client
        .describe_secret()
        .secret_id(create_response_5.arn().unwrap())
        .send()
        .await
        .unwrap();

    assert!(secret_5_describe.last_accessed_date().is_none());
}

/// Tests that BatchGetSecretValue finds all the expected secrets
/// when searching using a filter for description
#[tokio::test]
async fn test_batch_get_secret_value_find_by_filter_description() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .description("test-description-1")
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .description("test-description-2")
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .description("test-description-3")
        .send()
        .await
        .unwrap();

    let _create_response_4 = client
        .create_secret()
        .name("should-not-match-test-4")
        .description("should-not-mach-test-description")
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let _create_response_5 = client
        .create_secret()
        .name("test-5")
        .secret_string("test-5")
        .description("TEST-description-5")
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .filters(
            Filter::builder()
                .key(aws_sdk_secretsmanager::types::FilterNameStringType::Description)
                .values("test-")
                .build(),
        )
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_3.arn());
    assert_eq!(secret_1.name(), create_response_3.name());
    assert_eq!(secret_1.version_id(), create_response_3.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-3"));

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());
    assert_eq!(secret_2.name(), create_response_2.name());
    assert_eq!(secret_2.version_id(), create_response_2.version_id());
    assert_eq!(secret_2.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_2.secret_binary(), None);
    assert_eq!(secret_2.secret_string(), Some("test-2"));

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_1.arn());
    assert_eq!(secret_3.name(), create_response_1.name());
    assert_eq!(secret_3.version_id(), create_response_1.version_id());
    assert_eq!(secret_3.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_3.secret_binary(), None);
    assert_eq!(secret_3.secret_string(), Some("test-1"));
}

/// Tests that BatchGetSecretValue finds all the expected secrets
/// when searching using a filter for tag key
#[tokio::test]
async fn test_batch_get_secret_value_find_by_filter_tag_key() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .tags(
            Tag::builder()
                .key("test-tag-1")
                .value("test-value-1")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .tags(
            Tag::builder()
                .key("test-tag-2")
                .value("test-value-2")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .tags(
            Tag::builder()
                .key("test-tag-3")
                .value("test-value-3")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let _create_response_4 = client
        .create_secret()
        .name("should-not-match-test-4")
        .tags(
            Tag::builder()
                .key("should-not-match-test-tag-4")
                .value("test-value-4")
                .build(),
        )
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let _create_response_5 = client
        .create_secret()
        .name("test-5")
        .secret_string("test-5")
        .tags(
            Tag::builder()
                .key("TEST-tag-5")
                .value("test-value-5")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .filters(
            Filter::builder()
                .key(aws_sdk_secretsmanager::types::FilterNameStringType::TagKey)
                .values("test-")
                .build(),
        )
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_3.arn());
    assert_eq!(secret_1.name(), create_response_3.name());
    assert_eq!(secret_1.version_id(), create_response_3.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-3"));

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());
    assert_eq!(secret_2.name(), create_response_2.name());
    assert_eq!(secret_2.version_id(), create_response_2.version_id());
    assert_eq!(secret_2.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_2.secret_binary(), None);
    assert_eq!(secret_2.secret_string(), Some("test-2"));

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_1.arn());
    assert_eq!(secret_3.name(), create_response_1.name());
    assert_eq!(secret_3.version_id(), create_response_1.version_id());
    assert_eq!(secret_3.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_3.secret_binary(), None);
    assert_eq!(secret_3.secret_string(), Some("test-1"));
}

/// Tests that BatchGetSecretValue finds all the expected secrets
/// when searching using a filter for tag value
#[tokio::test]
async fn test_batch_get_secret_value_find_by_filter_tag_value() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .tags(
            Tag::builder()
                .key("test-tag-1")
                .value("test-value-1")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .tags(
            Tag::builder()
                .key("test-tag-2")
                .value("test-value-2")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .tags(
            Tag::builder()
                .key("test-tag-3")
                .value("test-value-3")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let _create_response_4 = client
        .create_secret()
        .name("should-not-match-test-4")
        .tags(
            Tag::builder()
                .key("should-not-match-test-tag-4")
                .value("should-not-match-test-value-4")
                .build(),
        )
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let _create_response_5 = client
        .create_secret()
        .name("test-5")
        .secret_string("test-5")
        .tags(
            Tag::builder()
                .key("test-tag-5")
                .value("TEST-value-5")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .filters(
            Filter::builder()
                .key(aws_sdk_secretsmanager::types::FilterNameStringType::TagValue)
                .values("test-")
                .build(),
        )
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_3.arn());
    assert_eq!(secret_1.name(), create_response_3.name());
    assert_eq!(secret_1.version_id(), create_response_3.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-3"));

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());
    assert_eq!(secret_2.name(), create_response_2.name());
    assert_eq!(secret_2.version_id(), create_response_2.version_id());
    assert_eq!(secret_2.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_2.secret_binary(), None);
    assert_eq!(secret_2.secret_string(), Some("test-2"));

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_1.arn());
    assert_eq!(secret_3.name(), create_response_1.name());
    assert_eq!(secret_3.version_id(), create_response_1.version_id());
    assert_eq!(secret_3.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_3.secret_binary(), None);
    assert_eq!(secret_3.secret_string(), Some("test-1"));
}

/// Prefixing a filter value with ! should invert the filter to instead exclude the value
#[tokio::test]
async fn test_batch_get_secret_value_negation_filter() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .tags(
            Tag::builder()
                .key("test-tag-1")
                .value("test-value-1")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .tags(
            Tag::builder()
                .key("test-tag-2")
                .value("test-value-2")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .tags(
            Tag::builder()
                .key("test-tag-3")
                .value("test-value-3")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let _create_response_4 = client
        .create_secret()
        .name("test-4")
        .secret_string("test-4")
        .tags(
            Tag::builder()
                .key("test-tag-4")
                .value("test-value-4")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .filters(
            Filter::builder()
                .key(aws_sdk_secretsmanager::types::FilterNameStringType::TagValue)
                .values("test-")
                .build(),
        )
        // Exclude test-4 from the results
        .filters(
            Filter::builder()
                .key(aws_sdk_secretsmanager::types::FilterNameStringType::TagValue)
                .values("!test-value-4")
                .build(),
        )
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_3.arn());
    assert_eq!(secret_1.name(), create_response_3.name());
    assert_eq!(secret_1.version_id(), create_response_3.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-3"));

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());
    assert_eq!(secret_2.name(), create_response_2.name());
    assert_eq!(secret_2.version_id(), create_response_2.version_id());
    assert_eq!(secret_2.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_2.secret_binary(), None);
    assert_eq!(secret_2.secret_string(), Some("test-2"));

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_1.arn());
    assert_eq!(secret_3.name(), create_response_1.name());
    assert_eq!(secret_3.version_id(), create_response_1.version_id());
    assert_eq!(secret_3.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_3.secret_binary(), None);
    assert_eq!(secret_3.secret_string(), Some("test-1"));
}

/// Tests that BatchGetSecretValue finds all the expected secrets
/// when searching using a filter for any
#[tokio::test]
async fn test_batch_get_secret_value_find_by_filter_all() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .tags(
            Tag::builder()
                .key("not-match-by-key-test-tag-1")
                .value("not-match-by-value-test-value-1")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("not-match-by-name-test-2")
        .secret_string("test-2")
        .description("test-2-match-by-description")
        .tags(
            Tag::builder()
                .key("not-match-key-test-tag-2")
                .value("not-match-value-test-value-2")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("not-match-by-name-test-3")
        .secret_string("test-3")
        .tags(
            Tag::builder()
                .key("test-tag-3")
                .value("not-match-by-value-test-value-3")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_4 = client
        .create_secret()
        .name("not-match-by-name-test-4")
        .secret_string("test-4")
        .tags(
            Tag::builder()
                .key("not-match-by-key-test-tag-4")
                .value("test-value-4")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let _create_response_5 = client
        .create_secret()
        .name("should-not-match-test-5")
        .tags(
            Tag::builder()
                .key("should-not-match-test-tag-5")
                .value("should-not-match-test-value-5")
                .build(),
        )
        .secret_string("test-5")
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .filters(
            Filter::builder()
                .key(aws_sdk_secretsmanager::types::FilterNameStringType::All)
                .values("test-")
                .build(),
        )
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 4);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_4.arn());
    assert_eq!(secret_1.name(), create_response_4.name());
    assert_eq!(secret_1.version_id(), create_response_4.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-4"));

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_3.arn());
    assert_eq!(secret_2.name(), create_response_3.name());
    assert_eq!(secret_2.version_id(), create_response_3.version_id());
    assert_eq!(secret_2.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_2.secret_binary(), None);
    assert_eq!(secret_2.secret_string(), Some("test-3"));

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_2.arn());
    assert_eq!(secret_3.name(), create_response_2.name());
    assert_eq!(secret_3.version_id(), create_response_2.version_id());
    assert_eq!(secret_3.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_3.secret_binary(), None);
    assert_eq!(secret_3.secret_string(), Some("test-2"));

    let secret_4 = secret_values.next().unwrap();
    assert_eq!(secret_4.arn(), create_response_1.arn());
    assert_eq!(secret_4.name(), create_response_1.name());
    assert_eq!(secret_4.version_id(), create_response_1.version_id());
    assert_eq!(secret_4.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_4.secret_binary(), None);
    assert_eq!(secret_4.secret_string(), Some("test-1"));
}

/// Tests that the expected error is present when a secret is missing
#[tokio::test]
async fn test_batch_get_secret_value_find_missing_secrets() {
    let (client, _server) = test_server().await;

    let create_response_4 = client
        .create_secret()
        .name("test-4")
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .secret_id_list("test-1")
        .secret_id_list("test-2")
        .secret_id_list("arn:aws:secretsmanager:us-west-2:1:secret:test-test")
        .secret_id_list("test-4")
        .send()
        .await
        .unwrap();

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 1);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_4.arn());
    assert_eq!(secret_1.name(), create_response_4.name());
    assert_eq!(secret_1.version_id(), create_response_4.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-4"));

    let errors = secrets.errors();
    assert_eq!(errors.len(), 3);

    let mut errors = errors.iter();
    let error_1 = errors.next().unwrap();
    assert_eq!(error_1.secret_id(), Some("test-1"));
    assert_eq!(error_1.error_code(), Some("ResourceNotFoundException"));

    let error_2 = errors.next().unwrap();
    assert_eq!(error_2.secret_id(), Some("test-2"));
    assert_eq!(error_2.error_code(), Some("ResourceNotFoundException"));

    let error_3 = errors.next().unwrap();
    assert_eq!(
        error_3.secret_id(),
        Some("arn:aws:secretsmanager:us-west-2:1:secret:test-test")
    );
    assert_eq!(error_3.error_code(), Some("ResourceNotFoundException"));
}

/// Tests that the default pagination behavior is working
#[tokio::test]
async fn test_batch_get_secret_value_default_pagination() {
    let (client, _server) = test_server().await;

    let page_size = 20;
    let pages = 3;
    let items_to_make = page_size * pages;

    let mut created_secrets: Vec<CreateSecretOutput> = Vec::new();

    for i in 0..items_to_make {
        let secret = client
            .create_secret()
            .name(format!("test-{i}"))
            .secret_string(format!("test-{i}"))
            .send()
            .await
            .unwrap();
        created_secrets.push(secret);
    }

    created_secrets.reverse();

    let mut next_token: Option<String> = None;

    for page in 0..pages {
        let secrets = client
            .batch_get_secret_value()
            .filters(
                Filter::builder()
                    .key(aws_sdk_secretsmanager::types::FilterNameStringType::Name)
                    .values("test-")
                    .build(),
            )
            .set_next_token(next_token)
            .send()
            .await
            .unwrap();

        for i in 0..page_size {
            let secret = secrets.secret_values().get(i).unwrap();
            let created_secret = created_secrets.get(i + (page_size * page)).unwrap();

            // Make sure the versions match
            assert_eq!(secret.arn(), created_secret.arn());
            assert_eq!(secret.version_id(), created_secret.version_id());
        }

        if page < pages - 1 {
            assert_eq!(
                secrets.next_token(),
                Some(format!("{}:{}", page_size, page + 1).as_str())
            );

            next_token = secrets.next_token().map(|value| value.to_string());
        } else {
            // Should have nothing more
            assert_eq!(secrets.next_token(), None);
            next_token = None;
        }
    }
}

/// Tests that pagination is working with a custom results size
#[tokio::test]
async fn test_batch_get_secret_value_custom_pagination() {
    let (client, _server) = test_server().await;

    let page_size = 5;
    let pages = 3;
    let items_to_make = page_size * pages;

    let mut created_secrets: Vec<CreateSecretOutput> = Vec::new();

    for i in 0..items_to_make {
        let secret = client
            .create_secret()
            .name(format!("test-{i}"))
            .secret_string(format!("test-{i}"))
            .send()
            .await
            .unwrap();
        created_secrets.push(secret);
    }

    created_secrets.reverse();

    let mut next_token: Option<String> = None;

    for page in 0..pages {
        let secrets = client
            .batch_get_secret_value()
            .filters(
                Filter::builder()
                    .key(aws_sdk_secretsmanager::types::FilterNameStringType::Name)
                    .values("test-")
                    .build(),
            )
            .max_results(page_size as i32)
            .set_next_token(next_token)
            .send()
            .await
            .unwrap();

        for i in 0..page_size {
            let secret = secrets.secret_values().get(i).unwrap();
            let created_secret = created_secrets.get(i + (page_size * page)).unwrap();

            // Make sure the versions match
            assert_eq!(secret.arn(), created_secret.arn());
            assert_eq!(secret.version_id(), created_secret.version_id());
        }

        if page < pages - 1 {
            assert_eq!(
                secrets.next_token(),
                Some(format!("{}:{}", page_size, page + 1).as_str())
            );

            next_token = secrets.next_token().map(|value| value.to_string());
        } else {
            // Should have nothing more
            assert_eq!(secrets.next_token(), None);
            next_token = None;
        }
    }
}

/// Tests that results don't include secrets that are scheduled
/// for deletion
#[tokio::test]
async fn test_batch_get_secret_value_no_scheduled_deletion() {
    let (client, _server) = test_server().await;

    let create_response_1 = client
        .create_secret()
        .name("test-1")
        .secret_string("test-1")
        .tags(
            Tag::builder()
                .key("test-tag-1")
                .value("test-value-1")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_2 = client
        .create_secret()
        .name("test-2")
        .secret_string("test-2")
        .tags(
            Tag::builder()
                .key("test-tag-2")
                .value("test-value-2")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_3 = client
        .create_secret()
        .name("test-3")
        .secret_string("test-3")
        .tags(
            Tag::builder()
                .key("test-tag-3")
                .value("test-value-3")
                .build(),
        )
        .send()
        .await
        .unwrap();

    let create_response_4 = client
        .create_secret()
        .name("test-4")
        .tags(
            Tag::builder()
                .key("test-tag-4")
                .value("test-value-4")
                .build(),
        )
        .secret_string("test-4")
        .send()
        .await
        .unwrap();

    client
        .delete_secret()
        .secret_id(create_response_4.arn().unwrap())
        .send()
        .await
        .unwrap();

    let secrets = client
        .batch_get_secret_value()
        .filters(
            Filter::builder()
                .key(aws_sdk_secretsmanager::types::FilterNameStringType::TagValue)
                .values("test-")
                .build(),
        )
        .send()
        .await
        .unwrap();

    assert!(secrets.errors().is_empty());

    let secret_values = secrets.secret_values();
    assert_eq!(secret_values.len(), 3);

    let mut secret_values = secret_values.iter();

    let secret_1 = secret_values.next().unwrap();
    assert_eq!(secret_1.arn(), create_response_3.arn());
    assert_eq!(secret_1.name(), create_response_3.name());
    assert_eq!(secret_1.version_id(), create_response_3.version_id());
    assert_eq!(secret_1.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_1.secret_binary(), None);
    assert_eq!(secret_1.secret_string(), Some("test-3"));

    let secret_2 = secret_values.next().unwrap();
    assert_eq!(secret_2.arn(), create_response_2.arn());
    assert_eq!(secret_2.name(), create_response_2.name());
    assert_eq!(secret_2.version_id(), create_response_2.version_id());
    assert_eq!(secret_2.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_2.secret_binary(), None);
    assert_eq!(secret_2.secret_string(), Some("test-2"));

    let secret_3 = secret_values.next().unwrap();
    assert_eq!(secret_3.arn(), create_response_1.arn());
    assert_eq!(secret_3.name(), create_response_1.name());
    assert_eq!(secret_3.version_id(), create_response_1.version_id());
    assert_eq!(secret_3.version_stages(), &["AWSCURRENT".to_string()]);
    assert_eq!(secret_3.secret_binary(), None);
    assert_eq!(secret_3.secret_string(), Some("test-1"));
}

/// Tests that specifying neither filters nor secret ids should
/// be an error
#[tokio::test]
async fn test_batch_get_secret_value_no_types_error() {
    let (client, _server) = test_server().await;

    let get_error = client.batch_get_secret_value().send().await.unwrap_err();

    let get_error = match get_error {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: InvalidRequestException = match get_error.into_err() {
        BatchGetSecretValueError::InvalidRequestException(error) => error,
        error => panic!("expected BatchGetSecretValueError::InvalidRequestException got {error:?}"),
    };
}

/// Tests that specifying both filters and secret ids should
/// be an error
#[tokio::test]
async fn test_batch_get_secret_value_both_types_error() {
    let (client, _server) = test_server().await;

    let get_error = client
        .batch_get_secret_value()
        .secret_id_list("test-1")
        .filters(
            Filter::builder()
                .key(aws_sdk_secretsmanager::types::FilterNameStringType::All)
                .values("test")
                .build(),
        )
        .send()
        .await
        .unwrap_err();

    let get_error = match get_error {
        SdkError::ServiceError(error) => error,
        error => panic!("expected SdkError::ServiceError got {error:?}"),
    };

    let _exception: InvalidRequestException = match get_error.into_err() {
        BatchGetSecretValueError::InvalidRequestException(error) => error,
        error => panic!("expected BatchGetSecretValueError::InvalidRequestException got {error:?}"),
    };
}

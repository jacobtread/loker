use crate::common::{
    TestServer, start_test_server, test_memory_database, test_sdk_config, test_server,
};
use aws_credential_types::Credentials;

mod common;

/// Tests that matching credentials will succeed
#[tokio::test]
async fn test_valid_credentials_success() {
    let (client, _server) = test_server().await;

    client
        .create_secret()
        .name("test")
        .secret_string("test")
        .send()
        .await
        .unwrap();
}

/// Tests that an invalid access key ID will fail
#[tokio::test]
async fn test_invalid_access_key_id_credentials_failure() {
    let db = test_memory_database().await;

    let (server_address, abort_handle) = start_test_server(
        db.clone(),
        Credentials::new("TEST", "test", None, None, "test"),
    )
    .await;

    let sdk_config = test_sdk_config(
        &format!("http://{server_address}/"),
        Credentials::new("TEST_THAT_DOES_NOT_MATCH", "test", None, None, "test"),
    );
    let client = aws_sdk_secretsmanager::Client::new(&sdk_config);

    let _server = TestServer { abort_handle, db };

    let err = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .send()
        .await
        .unwrap_err();

    assert!(
        err.as_service_error()
            .is_some_and(|value| value.meta().code() == Some("InvalidClientTokenId"))
    );
}

/// Tests that an invalid secret key value will fail
#[tokio::test]
async fn test_invalid_access_key_secret_credentials_failure() {
    let db = test_memory_database().await;

    let (server_address, abort_handle) = start_test_server(
        db.clone(),
        Credentials::new("TEST_THAT_DOES_NOT_MATCH", "test", None, None, "test"),
    )
    .await;

    let sdk_config = test_sdk_config(
        &format!("http://{server_address}/"),
        Credentials::new(
            "TEST_THAT_DOES_NOT_MATCH",
            "test_not_matching",
            None,
            None,
            "test",
        ),
    );
    let client = aws_sdk_secretsmanager::Client::new(&sdk_config);

    let _server = TestServer { abort_handle, db };

    let err = client
        .create_secret()
        .name("test")
        .secret_string("test")
        .send()
        .await
        .unwrap_err();

    assert!(
        err.as_service_error()
            .is_some_and(|value| value.meta().code() == Some("SignatureDoesNotMatch"))
    );
}

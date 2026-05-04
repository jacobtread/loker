use std::net::SocketAddr;

use axum::{Extension, Router, routing::post_service};
use loker::{
    database::{DbHandle, initialize_database},
    handlers::{self},
    middleware::aws_sig_v4::AwsSigV4AuthLayer,
};

use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_sdk_secretsmanager::config::{Credentials, SharedCredentialsProvider};
use tokio::task::AbortHandle;
use tokio_rusqlite::Connection;

/// Create an AWS sdk config for use in tests
#[allow(dead_code)]
pub fn test_sdk_config(endpoint_url: &str, credentials: Credentials) -> SdkConfig {
    SdkConfig::builder()
        .behavior_version(BehaviorVersion::v2026_01_12())
        .region(Region::from_static("us-east-1"))
        .endpoint_url(endpoint_url)
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .build()
}

#[allow(dead_code)]
pub struct TestServer {
    pub db: Connection,
    pub abort_handle: AbortHandle,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}

#[allow(dead_code)]
pub async fn test_memory_database() -> DbHandle {
    let db = Connection::open_in_memory().await.unwrap();
    db.call_unwrap(move |db| {
        db.pragma_update(None, "case_sensitive_like", true).unwrap();
        initialize_database(db).unwrap();
    })
    .await;
    db
}

#[allow(dead_code)]
pub async fn start_test_server(
    db: Connection,
    credentials: Credentials,
) -> (SocketAddr, AbortHandle) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_address = listener.local_addr().unwrap();

    let abort_handle = tokio::spawn(async move {
        let handlers = handlers::create_handlers();
        let handlers_service = handlers.into_service();
        let app = Router::new()
            .route_service("/", post_service(handlers_service))
            .layer(AwsSigV4AuthLayer::new(credentials))
            .layer(Extension(db));

        axum::serve(listener, app).await.unwrap();
    })
    .abort_handle();

    (server_address, abort_handle)
}

#[allow(dead_code)]
pub async fn test_server() -> (aws_sdk_secretsmanager::Client, TestServer) {
    let db = test_memory_database().await;
    let credentials = Credentials::for_tests();

    let (server_address, abort_handle) = start_test_server(db.clone(), credentials.clone()).await;

    let sdk_config = test_sdk_config(&format!("http://{server_address}/"), credentials);
    let client = aws_sdk_secretsmanager::Client::new(&sdk_config);

    (client, TestServer { abort_handle, db })
}

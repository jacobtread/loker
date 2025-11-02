use axum::{Extension, Router, routing::post_service};
use loker::{
    database::{DbPool, initialize_database},
    handlers::{self},
    middleware::aws_sig_v4::AwsSigV4AuthLayer,
};
use sqlx::sqlite::SqlitePoolOptions;

use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_sdk_secretsmanager::config::{Credentials, SharedCredentialsProvider};
use tokio::task::AbortHandle;

/// Create an AWS sdk config for use in tests
#[allow(dead_code)]
pub fn test_sdk_config(endpoint_url: &str, credentials: Credentials) -> SdkConfig {
    SdkConfig::builder()
        .behavior_version(BehaviorVersion::v2025_08_07())
        .region(Region::from_static("us-east-1"))
        .endpoint_url(endpoint_url)
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .build()
}

#[allow(dead_code)]
pub struct TestServer {
    sdk_config: SdkConfig,
    pub db: DbPool,
    handle: AbortHandle,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[allow(dead_code)]
async fn memory_database() -> DbPool {
    let pool = SqlitePoolOptions::new()
        .after_connect(move |connection, _metadata| {
            Box::pin(async move {
                // Enable case sensitive LIKE
                sqlx::query("PRAGMA case_sensitive_like = ON;")
                    .execute(connection)
                    .await?;

                Ok(())
            })
        })
        .connect("sqlite::memory:")
        .await
        .unwrap();
    initialize_database(&pool).await.unwrap();
    pool
}

#[allow(dead_code)]
pub async fn test_server() -> (aws_sdk_secretsmanager::Client, TestServer) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let db = memory_database().await;
    let server_address = listener.local_addr().unwrap();
    let harness_db = db.clone();

    let credentials = Credentials::for_tests();
    let credentials_client_copy = credentials.clone();

    let abort_handle = tokio::spawn(async move {
        let handlers = handlers::create_handlers();
        let handlers_service = handlers.into_service();
        let app = Router::new()
            .route_service("/", post_service(handlers_service))
            .layer(AwsSigV4AuthLayer::new(credentials))
            .layer(Extension(db.clone()));

        axum::serve(listener, app).await.unwrap();
    })
    .abort_handle();

    let sdk_config = test_sdk_config(
        &format!("http://{server_address}/"),
        credentials_client_copy,
    );
    let client = aws_sdk_secretsmanager::Client::new(&sdk_config);

    (
        client,
        TestServer {
            sdk_config,
            handle: abort_handle,
            db: harness_db,
        },
    )
}

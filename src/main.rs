#![forbid(unsafe_code)]

use crate::{
    background::perform_background_tasks, config::Config, middleware::aws_sig_v4::AwsSigV4AuthLayer,
};
use axum::{Extension, Router, http::StatusCode, routing::post_service};
use axum_server::tls_rustls::RustlsConfig;
use std::{error::Error, net::SocketAddr};
use tower_http::trace::TraceLayer;

pub mod database;
pub mod middleware;

mod background;
mod config;
mod handlers;
mod logging;
mod utils;

fn main() -> Result<(), Box<dyn Error>> {
    _ = dotenvy::dotenv();

    logging::init_logging();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(async move {
            if let Err(error) = server().await {
                tracing::error!(?error, message = %error, "error running server");
                return Err(error);
            }

            Ok(())
        })
}

async fn server() -> Result<(), Box<dyn Error>> {
    let config = match Config::from_env() {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(?error, "failed to load configuration");
            return Err(error.into());
        }
    };

    // Setup database
    let db = database::create_database(config.encryption_key, config.database_path).await?;

    // Setup the handlers
    let handlers = handlers::create_handlers();
    let handlers_service = handlers.into_service();

    // Setup router
    let app = Router::new()
        .route_service("/", post_service(handlers_service))
        .layer(AwsSigV4AuthLayer::new(config.credentials))
        .route("/health", axum::routing::get(health))
        .layer(Extension(db.clone()))
        .layer(TraceLayer::new_for_http());

    // Development mode CORS access for local browser testing
    #[cfg(debug_assertions)]
    let app = app.layer(tower_http::cors::CorsLayer::very_permissive());

    // Spawn the background task runner
    tokio::spawn(perform_background_tasks(db.clone()));

    let handle = axum_server::Handle::default();

    // Handle graceful shutdown on CTRL+C
    tokio::spawn({
        let handle = handle.clone();
        async move {
            _ = tokio::signal::ctrl_c().await;
            handle.graceful_shutdown(None);
        }
    });

    tracing::debug!("starting server on {}", config.server_address);

    if config.use_https {
        serve_https(
            app,
            handle,
            config.server_address,
            config.certificate_path,
            config.private_key_path,
        )
        .await?;
    } else {
        serve_http(app, handle, config.server_address).await?;
    }

    Ok(())
}

/// Health check route
async fn health() -> StatusCode {
    StatusCode::OK
}

/// Serve the app over HTTPS
async fn serve_https(
    app: Router,
    handle: axum_server::Handle,
    server_address: SocketAddr,
    certificate_path: String,
    private_key_path: String,
) -> Result<(), Box<dyn Error>> {
    if rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .is_err()
    {
        tracing::error!("failed install default crypto provider");
        return Err(std::io::Error::other("failed to install default crypto provider").into());
    }

    let config = match RustlsConfig::from_pem_file(certificate_path, private_key_path).await {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(?error, "failed to initialize https config");
            return Err(error.into());
        }
    };

    axum_server::bind_rustls(server_address, config)
        .handle(handle)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Serve the app over HTTP
async fn serve_http(
    app: Router,
    handle: axum_server::Handle,
    server_address: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    axum_server::bind(server_address)
        .handle(handle)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

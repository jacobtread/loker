use aws_credential_types::Credentials;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use thiserror::Error;

/// Default server address when not specified (HTTP)
const DEFAULT_SERVER_ADDRESS_HTTP: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080));

/// Default server address when not specified (HTTPS)
const DEFAULT_SERVER_ADDRESS_HTTPS: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8443));

pub struct Config {
    /// Encryption key to encrypt and decrypt the database
    pub encryption_key: String,
    /// Path to the server database file
    pub database_path: String,

    /// Server address to bind against
    pub server_address: SocketAddr,

    /// Whether to use HTTPS instead of HTTP
    pub use_https: bool,
    /// Path to the HTTPS certificate file
    pub certificate_path: String,
    /// Path to the HTTPS private key file
    pub private_key_path: String,

    /// Credentials for AWS SigV4
    pub credentials: Credentials,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Must specify SM_ENCRYPTION_KEY environment variable")]
    MissingEncryptionKey,

    #[error("Must specify SM_ACCESS_KEY_ID environment variable")]
    MissingAccessKeyId,

    #[error("Must specify SM_ACCESS_KEY_SECRET environment variable")]
    MissingAccessKeySecret,

    #[error("SM_USE_HTTPS must be either true or false")]
    InvalidUseHttps,
}

impl Config {
    /// Load the config from the environment variables
    pub fn from_env() -> Result<Config, ConfigError> {
        let encryption_key =
            std::env::var("SM_ENCRYPTION_KEY").map_err(|_| ConfigError::MissingEncryptionKey)?;

        let access_key_id =
            std::env::var("SM_ACCESS_KEY_ID").map_err(|_| ConfigError::MissingAccessKeyId)?;

        let access_key_secret = std::env::var("SM_ACCESS_KEY_SECRET")
            .map_err(|_| ConfigError::MissingAccessKeySecret)?;

        let credentials = Credentials::new(
            access_key_id,
            access_key_secret,
            None,
            None,
            "sm-credentials",
        );

        let database_path =
            std::env::var("SM_DATABASE_PATH").unwrap_or_else(|_| "secrets.db".to_string());

        let use_https = match std::env::var("SM_USE_HTTPS") {
            Ok(value) => value
                .parse::<bool>()
                .map_err(|_| ConfigError::InvalidUseHttps)?,
            Err(_) => false,
        };

        let server_address = std::env::var("SM_SERVER_ADDRESS")
            .ok()
            .and_then(|value| value.parse::<SocketAddr>().ok())
            .unwrap_or(if use_https {
                DEFAULT_SERVER_ADDRESS_HTTPS
            } else {
                DEFAULT_SERVER_ADDRESS_HTTP
            });

        let certificate_path = match std::env::var("SM_HTTPS_CERTIFICATE_PATH") {
            Ok(value) => value,
            Err(_) => "sm.cert.pem".to_string(),
        };

        let private_key_path = match std::env::var("SM_HTTPS_PRIVATE_KEY_PATH") {
            Ok(value) => value,
            Err(_) => "sm.key.pem".to_string(),
        };

        Ok(Config {
            encryption_key,
            database_path,
            use_https,
            server_address,
            certificate_path,
            private_key_path,
            credentials,
        })
    }
}

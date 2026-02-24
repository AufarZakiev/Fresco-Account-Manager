use std::net::SocketAddr;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FamConfig {
    pub database_url: String,
    pub listen_addr: SocketAddr,
    pub server_name: String,
    pub repeat_sec: u32,
    pub private_key_path: String,
    pub public_key_path: String,
}

impl FamConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = std::env::var("FAM_DATABASE_URL")
            .map_err(|_| ConfigError::Missing("FAM_DATABASE_URL"))?;

        let listen_addr = std::env::var("FAM_LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
            .parse::<SocketAddr>()
            .map_err(|_| ConfigError::Invalid("FAM_LISTEN_ADDR"))?;

        let server_name = std::env::var("FAM_SERVER_NAME")
            .unwrap_or_else(|_| "Fresco Account Manager".to_string());

        let repeat_sec = std::env::var("FAM_REPEAT_SEC")
            .unwrap_or_else(|_| "86400".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::Invalid("FAM_REPEAT_SEC"))?;

        let private_key_path = std::env::var("FAM_PRIVATE_KEY_PATH")
            .unwrap_or_else(|_| "keys/private.pem".to_string());

        let public_key_path =
            std::env::var("FAM_PUBLIC_KEY_PATH").unwrap_or_else(|_| "keys/public.pem".to_string());

        tracing::debug!(
            repeat_sec,
            %private_key_path,
            %public_key_path,
            "loaded config"
        );

        Ok(Self {
            database_url,
            listen_addr,
            server_name,
            repeat_sec,
            private_key_path,
            public_key_path,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("invalid value for environment variable: {0}")]
    Invalid(&'static str),
}

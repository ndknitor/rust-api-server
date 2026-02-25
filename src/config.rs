use std::env;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_ttl: u64,
    pub cors_origin: String,
    pub rust_log: String,
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidPort(std::num::ParseIntError),
    InvalidJwtTtl(std::num::ParseIntError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidPort(err) => write!(f, "invalid PORT: {err}"),
            ConfigError::InvalidJwtTtl(err) => write!(f, "invalid JWT_TTL: {err}"),
        }
    }
}

impl Error for ConfigError {}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(ConfigError::InvalidPort)?;

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "key".to_string());
        let jwt_ttl = env::var("JWT_TTL")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .map_err(ConfigError::InvalidJwtTtl)?;

        let cors_origin =
            env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());

        let rust_log =
            env::var("RUST_LOG").unwrap_or_else(|_| "info,rust_api_server=debug".to_string());

        Ok(Self {
            host,
            port,
            jwt_secret,
            jwt_ttl,
            cors_origin,
            rust_log,
        })
    }
}

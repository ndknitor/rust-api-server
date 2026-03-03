use std::sync::{Arc, OnceLock};

use sea_orm::DatabaseConnection;
use thiserror::Error;

use crate::config::{Config, ConfigError};
use crate::services::jwt::{JWTService, JWTServiceImpl};

#[derive(Debug, Error)]
pub enum InjectError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("inject not initialized")]
    NotInitialized,
}

pub trait InjectFactory {
    fn config(&self) -> Result<Arc<Config>, InjectError>;
    fn jwt_service(&self) -> Result<Arc<dyn JWTService>, InjectError>;
    fn database(&self) -> Result<Arc<DatabaseConnection>, InjectError>;
}

pub struct InjectFactoryImpl {
    config: OnceLock<Arc<Config>>,
    jwt_service: OnceLock<Arc<dyn JWTService>>,
    database: OnceLock<Arc<DatabaseConnection>>,
}

impl InjectFactoryImpl {
    pub async fn init() -> Result<Self, InjectError> {
        let config = Arc::new(Config::from_env()?);

        let jwt_service: Arc<dyn JWTService> =
            Arc::new(JWTServiceImpl::new(config.jwt_secret.clone()));

        // Connect to database
        let db = sea_orm::Database::connect(&config.database_url).await?;
        let database = Arc::new(db);

        Ok(Self {
            config: OnceLock::from(config),
            jwt_service: OnceLock::from(jwt_service),
            database: OnceLock::from(database),
        })
    }
}

impl InjectFactory for InjectFactoryImpl {
    fn config(&self) -> Result<Arc<Config>, InjectError> {
        self.config
            .get()
            .cloned()
            .ok_or(InjectError::NotInitialized)
    }

    fn jwt_service(&self) -> Result<Arc<dyn JWTService>, InjectError> {
        self.jwt_service
            .get()
            .cloned()
            .ok_or(InjectError::NotInitialized)
    }

    fn database(&self) -> Result<Arc<DatabaseConnection>, InjectError> {
        self.database
            .get()
            .cloned()
            .ok_or(InjectError::NotInitialized)
    }
}

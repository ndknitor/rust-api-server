use std::sync::{Arc, OnceLock};

use crate::config::{Config, ConfigError};
use crate::services::jwt::{JWTService, JWTServiceImpl};

pub trait InjectFactory {
    fn config(&self) -> Result<Arc<Config>, ConfigError>;
    fn jwt_service(&self) -> Result<Arc<dyn JWTService>, ConfigError>;
}

pub struct InjectFactoryImpl;

static CONFIG_SINGLETON: OnceLock<Arc<Config>> = OnceLock::new();
static JWT_SERVICE_SINGLETON: OnceLock<Arc<dyn JWTService>> = OnceLock::new();

impl InjectFactoryImpl {
    pub fn new() -> Self {
        Self
    }
}

impl InjectFactory for InjectFactoryImpl {
    fn config(&self) -> Result<Arc<Config>, ConfigError> {
        if let Some(config) = CONFIG_SINGLETON.get() {
            return Ok(Arc::clone(config));
        }

        let config = Arc::new(Config::from_env()?);
        let _ = CONFIG_SINGLETON.set(Arc::clone(&config));
        Ok(config)
    }

    fn jwt_service(&self) -> Result<Arc<dyn JWTService>, ConfigError> {
        if let Some(service) = JWT_SERVICE_SINGLETON.get() {
            return Ok(Arc::clone(service));
        }

        let config = self.config()?;
        let service: Arc<dyn JWTService> = Arc::new(JWTServiceImpl::new(config.jwt_secret.clone()));
        let _ = JWT_SERVICE_SINGLETON.set(Arc::clone(&service));
        Ok(service)
    }
}

use crate::config::Config;
use crate::services::auth::{AuthServiceImpl, AuthServiceTrait};
use crate::services::seat::{SeatServiceImpl, SeatServiceTrait};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub trait InjectFactory: Send + Sync {
    /// Singleton: shared instance across all requests
    fn auth_service(&self) -> Arc<dyn AuthServiceTrait>;

    /// Scoped: new instance per request, shares db connection pool
    fn seat_service(&self) -> Box<dyn SeatServiceTrait>;
}

pub struct InjectFactoryImpl {
    db: DatabaseConnection,
    auth: Arc<dyn AuthServiceTrait>,
}

impl InjectFactoryImpl {
    pub fn new(db: DatabaseConnection, cfg: &Config) -> Self {
        let auth = Arc::new(AuthServiceImpl::new(
            cfg.jwt_secret.clone(),
            cfg.jwt_ttl,
            cfg.environment.clone(),
        ));
        Self { db, auth }
    }
}

impl InjectFactory for InjectFactoryImpl {
    fn auth_service(&self) -> Arc<dyn AuthServiceTrait> {
        Arc::clone(&self.auth)
    }

    fn seat_service(&self) -> Box<dyn SeatServiceTrait> {
        Box::new(SeatServiceImpl::new(self.db.clone()))
    }
}

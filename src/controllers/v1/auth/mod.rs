use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tonic::{Request, Response, Status};

pub mod errors;
pub mod login;
pub mod logout;

use crate::config::Config;
use crate::pb;
use crate::services::auth::Auth;
use crate::services::jwt::JWTService;

pub use errors::AuthControllerError;
pub use login::{login_cookie, login_jwt};
pub use logout::logout;

pub struct AuthState {
    pub config: Arc<Config>,
    pub jwt_service: Arc<dyn JWTService>,
    pub auth_service: Arc<dyn Auth>,
    pub db: Arc<DatabaseConnection>,
}

impl AuthState {
    pub fn new(
        config: Arc<Config>,
        jwt_service: Arc<dyn JWTService>,
        auth_service: Arc<dyn Auth>,
        db: Arc<DatabaseConnection>,
    ) -> Self {
        Self {
            config,
            jwt_service,
            auth_service,
            db,
        }
    }
}

#[tonic::async_trait]
impl pb::auth_service_server::AuthService for AuthState {
    async fn login_jwt(
        &self,
        request: Request<pb::LoginRequest>,
    ) -> Result<Response<pb::LoginJwtResponse>, Status> {
        login::grpc_login_jwt(self, request).await
    }

    async fn login_cookie(
        &self,
        request: Request<pb::LoginRequest>,
    ) -> Result<Response<pb::LoginResponse>, Status> {
        login::grpc_login_cookie(self, request).await
    }

    async fn logout(
        &self,
        request: Request<pb::LogoutRequest>,
    ) -> Result<Response<pb::LogoutResponse>, Status> {
        logout::grpc_logout(self, request).await
    }
}

pub fn build_auth_cookie(token: &str, ttl: u64) -> String {
    format!("auth_token={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={ttl}")
}

pub fn clear_auth_cookie() -> String {
    "auth_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".to_string()
}

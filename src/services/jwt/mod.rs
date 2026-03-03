mod jwt;
mod errors;

pub use jwt::JWTServiceImpl;
pub use errors::JWTServiceError;

use async_trait::async_trait;

#[async_trait]
pub trait JWTService: Send + Sync {
    async fn sign_token(
        &self,
        subject: String,
        ttl_seconds: u64,
        roles: Vec<String>,
        policies: Vec<String>,
    ) -> Result<String, JWTServiceError>;
}

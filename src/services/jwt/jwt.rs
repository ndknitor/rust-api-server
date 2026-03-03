use async_trait::async_trait;
use libs::jwt::{build_claims, encode_token};

pub use super::JWTServiceError;

pub struct JWTServiceImpl {
    secret: String,
}

impl JWTServiceImpl {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }
}

#[async_trait]
impl super::JWTService for JWTServiceImpl {
    async fn sign_token(
        &self,
        subject: String,
        ttl_seconds: u64,
        roles: Vec<String>,
        policies: Vec<String>,
    ) -> Result<String, JWTServiceError> {
        let claims = build_claims(subject, ttl_seconds, roles, policies);
        encode_token(&claims, self.secret.as_str()).map_err(|_| JWTServiceError::SignFailed)
    }
}

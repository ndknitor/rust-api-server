use sea_orm::DbErr;
use thiserror::Error;
use tonic::Status;

use crate::services::auth::AuthError;

#[derive(Debug, Error)]
pub enum AuthControllerError {
    #[error("auth error: {0}")]
    Auth(#[from] AuthError),

    #[error("jwt service error: {0}")]
    JwtService(#[from] crate::services::jwt::JWTServiceError),

    #[error("invalid cookie metadata")]
    InvalidCookie,

    #[error("validation error: {0}")]
    Validation(String),
}

impl From<DbErr> for AuthControllerError {
    fn from(err: DbErr) -> Self {
        AuthControllerError::Auth(AuthError::Database(err))
    }
}

impl From<AuthControllerError> for Status {
    fn from(err: AuthControllerError) -> Self {
        match err {
            AuthControllerError::Auth(e) => match e {
                AuthError::InvalidCredentials => Status::unauthenticated("invalid credentials"),
                AuthError::Database(_) => Status::internal("database error"),
            },
            AuthControllerError::JwtService(_) => Status::internal("failed to sign token"),
            AuthControllerError::InvalidCookie => Status::internal("invalid cookie metadata"),
            AuthControllerError::Validation(msg) => Status::invalid_argument(msg),
        }
    }
}

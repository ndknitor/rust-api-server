use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("database error: {0}")]
    Database(#[from] DbErr),
    #[error("invalid credentials")]
    InvalidCredentials,
}

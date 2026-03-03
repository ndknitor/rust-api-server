use axum::{
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::DbErr;
use serde::Serialize;
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

    #[error("validation error: {0:?}")]
    Validation(Vec<String>),
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    messages: Vec<String>,
}

impl From<DbErr> for AuthControllerError {
    fn from(err: DbErr) -> Self {
        AuthControllerError::Auth(AuthError::Database(err))
    }
}

pub fn extract_validation_messages(e: validator::ValidationErrors) -> Vec<String> {
    e.field_errors()
        .into_iter()
        .flat_map(|(field, errors)| {
            errors.iter().map(move |err| {
                err.message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("{}: validation failed", field))
            })
        })
        .collect()
}

// gRPC
impl From<AuthControllerError> for Status {
    fn from(err: AuthControllerError) -> Self {
        match err {
            AuthControllerError::Auth(e) => match e {
                AuthError::InvalidCredentials => Status::unauthenticated("invalid credentials"),
                AuthError::Database(_) => Status::internal("database error"),
            },
            AuthControllerError::JwtService(_) => Status::internal("failed to sign token"),
            AuthControllerError::InvalidCookie => Status::internal("invalid cookie metadata"),
            AuthControllerError::Validation(msgs) => Status::invalid_argument(msgs.join(", ")),
        }
    }
}

// HTTP
impl IntoResponse for AuthControllerError {
    fn into_response(self) -> axum::response::Response {
        let (status, messages) = match self {
            AuthControllerError::Auth(e) => match e {
                AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, vec!["invalid credentials".to_string()]),
                AuthError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, vec!["database error".to_string()]),
            },
            AuthControllerError::JwtService(_) => (StatusCode::INTERNAL_SERVER_ERROR, vec!["failed to sign token".to_string()]),
            AuthControllerError::InvalidCookie => (StatusCode::INTERNAL_SERVER_ERROR, vec!["invalid cookie metadata".to_string()]),
            AuthControllerError::Validation(msgs) => (StatusCode::BAD_REQUEST, msgs),
        };

        (status, Json(ErrorResponse { code: status.as_u16(), messages })).into_response()
    }
}

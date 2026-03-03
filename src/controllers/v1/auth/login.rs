use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use validator::Validate;

use crate::controllers::v1::auth::{AuthControllerError, AuthState};
use crate::pb;
use crate::services::jwt::JWTService;

use super::build_auth_cookie;

#[derive(Validate)]
pub struct LoginInput {
    #[validate(length(min = 1, message = "username is required"))]
    pub username: String,

    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub password: String,
}

// HTTP

pub async fn login_jwt(
    State(state): State<Arc<AuthState>>,
    Json(input): Json<pb::LoginRequest>,
) -> Result<Json<pb::LoginJwtResponse>, StatusCode> {
    let login_input = LoginInput {
        username: input.username.clone(),
        password: input.password.clone(),
    };
    login_input.validate().map_err(|e| map_validation_error(e))?;

    let user = state.auth_service.find_user_by_email_password(&state.db, &input.username, &input.password)
        .await
        .map_err(map_auth_error_http)?;

    let token = do_login(&state.jwt_service, &state.config, &user.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(pb::LoginJwtResponse {
        status: "ok".to_string(),
        token,
    }))
}

pub async fn login_cookie(
    State(state): State<Arc<AuthState>>,
    Json(input): Json<pb::LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let login_input = LoginInput {
        username: input.username.clone(),
        password: input.password.clone(),
    };
    login_input.validate().map_err(|e| map_validation_error(e))?;

    let user = state.auth_service.find_user_by_email_password(&state.db, &input.username, &input.password)
        .await
        .map_err(map_auth_error_http)?;

    let token = do_login(&state.jwt_service, &state.config, &user.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut headers = HeaderMap::new();
    let cookie = format!("auth_token={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}", state.config.jwt_ttl);
    let value = HeaderValue::from_str(&cookie).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    headers.insert(SET_COOKIE, value);

    Ok((headers, Json(pb::LoginResponse { status: "ok".to_string() })))
}

// gRPC handlers

pub async fn grpc_login_jwt(
    service: &AuthState,
    request: Request<pb::LoginRequest>,
) -> Result<Response<pb::LoginJwtResponse>, Status> {
    let input = request.into_inner();
    let login_input = LoginInput {
        username: input.username.clone(),
        password: input.password.clone(),
    };
    login_input.validate().map_err(|e| AuthControllerError::Validation(e.to_string()))?;

    let user = service.auth_service.find_user_by_email_password(&service.db, &input.username, &input.password)
        .await
        .map_err(AuthControllerError::from)?;

    let token = do_login_grpc(&service.jwt_service, &service.config, &user.email)
        .await?;

    Ok(Response::new(pb::LoginJwtResponse {
        status: "ok".to_string(),
        token,
    }))
}

pub async fn grpc_login_cookie(
    service: &AuthState,
    request: Request<pb::LoginRequest>,
) -> Result<Response<pb::LoginResponse>, Status> {
    let input = request.into_inner();
    let login_input = LoginInput {
        username: input.username.clone(),
        password: input.password.clone(),
    };
    login_input.validate().map_err(|e| AuthControllerError::Validation(e.to_string()))?;

    let user = service.auth_service.find_user_by_email_password(&service.db, &input.username, &input.password)
        .await
        .map_err(AuthControllerError::from)?;

    let token = do_login_grpc(&service.jwt_service, &service.config, &user.email)
        .await?;

    let cookie = build_auth_cookie(&token, service.config.jwt_ttl);
    let cookie_meta = tonic::metadata::MetadataValue::try_from(cookie.as_str())
        .map_err(|_| AuthControllerError::InvalidCookie)?;

    let mut response = Response::new(pb::LoginResponse {
        status: "ok".to_string(),
    });
    response.metadata_mut().insert("set-cookie", cookie_meta);

    Ok(response)
}

async fn do_login(jwt_service: &Arc<dyn JWTService>, config: &crate::config::Config, email: &str) -> Result<String, ()> {
    jwt_service.sign_token(
        email.to_string(),
        config.jwt_ttl,
        vec!["user".to_string()],
        vec!["protected.read".to_string()],
    )
    .await
    .map_err(|_| ())
}

async fn do_login_grpc(jwt_service: &Arc<dyn JWTService>, config: &crate::config::Config, email: &str) -> Result<String, AuthControllerError> {
    Ok(jwt_service.sign_token(
        email.to_string(),
        config.jwt_ttl,
        vec!["user".to_string()],
        vec!["protected.read".to_string()],
    ).await?)
}

fn map_auth_error_http(e: crate::services::auth::AuthError) -> StatusCode {
    match e {
        crate::services::auth::AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
        crate::services::auth::AuthError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn map_validation_error(_e: validator::ValidationErrors) -> StatusCode {
    StatusCode::BAD_REQUEST
}

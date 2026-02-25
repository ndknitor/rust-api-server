use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use std::sync::Arc;
use tonic::metadata::MetadataValue;
use tonic::{Request, Response, Status};

use crate::{config::Config, pb, services::jwt::JWTService};

pub struct AuthController {
    config: Arc<Config>,
    jwt_service: Arc<dyn JWTService>,
}

impl AuthController {
    pub fn new(config: Arc<Config>, jwt_service: Arc<dyn JWTService>) -> Self {
        Self {
            config,
            jwt_service,
        }
    }
}

#[tonic::async_trait]
impl pb::auth_service_server::AuthService for AuthController {
    async fn login_jwt(
        &self,
        request: Request<pb::LoginRequest>,
    ) -> Result<Response<pb::LoginJwtResponse>, Status> {
        let input = request.into_inner();
        ensure_credentials(&input.username, &input.password)
            .map_err(|_| Status::unauthenticated("invalid credentials"))?;

        let token = match sign_token(self.jwt_service.as_ref(), &self.config, &input.username) {
            Ok(token) => token,
            Err(_) => return Err(Status::internal("failed to sign token")),
        };

        Ok(Response::new(login_jwt_response(token)))
    }

    async fn login_cookie(
        &self,
        request: Request<pb::LoginRequest>,
    ) -> Result<Response<pb::LoginResponse>, Status> {
        let input = request.into_inner();
        ensure_credentials(&input.username, &input.password)
            .map_err(|_| Status::unauthenticated("invalid credentials"))?;

        let token = match sign_token(self.jwt_service.as_ref(), &self.config, &input.username) {
            Ok(token) => token,
            Err(_) => return Err(Status::internal("failed to sign token")),
        };

        let cookie = build_auth_cookie(&token, self.config.jwt_ttl);
        let cookie_meta = match MetadataValue::try_from(cookie.as_str()) {
            Ok(meta) => meta,
            Err(_) => return Err(Status::internal("invalid set-cookie metadata")),
        };

        let mut response = Response::new(login_response());
        response.metadata_mut().insert("set-cookie", cookie_meta);

        Ok(response)
    }

    async fn logout(
        &self,
        _request: Request<pb::LogoutRequest>,
    ) -> Result<Response<pb::LogoutResponse>, Status> {
        let cookie_meta = match MetadataValue::try_from(clear_auth_cookie().as_str()) {
            Ok(meta) => meta,
            Err(_) => return Err(Status::internal("invalid set-cookie metadata")),
        };

        let mut response = Response::new(logout_response());
        response.metadata_mut().insert("set-cookie", cookie_meta);

        Ok(response)
    }
}

pub async fn login_jwt_http(
    State(config): State<Arc<Config>>,
    Extension(jwt_service): Extension<Arc<dyn JWTService>>,
    Json(input): Json<pb::LoginRequest>,
) -> Result<Json<pb::LoginJwtResponse>, StatusCode> {
    ensure_credentials(&input.username, &input.password).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token = match sign_token(jwt_service.as_ref(), &config, &input.username) {
        Ok(token) => token,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(login_jwt_response(token)))
}

pub async fn login_cookie_http(
    State(config): State<Arc<Config>>,
    Extension(jwt_service): Extension<Arc<dyn JWTService>>,
    Json(input): Json<pb::LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    ensure_credentials(&input.username, &input.password).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token = match sign_token(jwt_service.as_ref(), &config, &input.username) {
        Ok(token) => token,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut headers = HeaderMap::new();
    let cookie = build_auth_cookie(&token, config.jwt_ttl);
    let value = match HeaderValue::from_str(&cookie) {
        Ok(value) => value,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    headers.insert(SET_COOKIE, value);

    Ok((headers, Json(login_response())))
}

pub async fn logout_http() -> Result<impl IntoResponse, StatusCode> {
    let mut headers = HeaderMap::new();
    let value = match HeaderValue::from_str(&clear_auth_cookie()) {
        Ok(value) => value,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    headers.insert(SET_COOKIE, value);

    Ok((headers, Json(logout_response())))
}

fn is_valid_user(username: &str, password: &str) -> bool {
    username == "admin" && password == "admin"
}

fn ensure_credentials(username: &str, password: &str) -> Result<(), ()> {
    match is_valid_user(username, password) {
        true => Ok(()),
        false => Err(()),
    }
}

fn sign_token(jwt_service: &dyn JWTService, config: &Config, username: &str) -> Result<String, ()> {
    jwt_service
        .sign_token(
            username.to_string(),
            config.jwt_ttl,
            vec!["user".to_string()],
            vec!["protected.read".to_string()],
        )
        .map_err(|_| ())
}

fn login_jwt_response(token: String) -> pb::LoginJwtResponse {
    pb::LoginJwtResponse {
        status: "ok".to_string(),
        token,
    }
}

fn login_response() -> pb::LoginResponse {
    pb::LoginResponse {
        status: "ok".to_string(),
    }
}

fn logout_response() -> pb::LogoutResponse {
    pb::LogoutResponse {
        status: "ok".to_string(),
    }
}

fn build_auth_cookie(token: &str, ttl: u64) -> String {
    format!("auth_token={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={ttl}")
}

fn clear_auth_cookie() -> String {
    "auth_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".to_string()
}

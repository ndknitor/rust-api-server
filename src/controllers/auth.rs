use crate::proto::auth::{AuthCookieResponse, AuthTokenResponse};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::HttpResponse;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::Response;

pub struct AuthController;

impl AuthController {
    pub fn expires_at(ttl_seconds: u64) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        (now + ttl_seconds) as i64
    }

    pub fn cookie_http(token: String, expires_at: i64) -> HttpResponse {
        let cookie = Cookie::build("auth_token", token)
            .path("/")
            .http_only(true)
            .secure(false)
            .same_site(SameSite::Lax)
            .finish();

        HttpResponse::Ok()
            .cookie(cookie)
            .json(AuthCookieResponse { expires_at })
    }

    pub fn token_http(token: String, expires_at: i64) -> HttpResponse {
        HttpResponse::Ok().json(AuthTokenResponse { token, expires_at })
    }

    pub fn cookie_grpc(expires_at: i64) -> Result<Response<AuthCookieResponse>, tonic::Status> {
        Ok(Response::new(AuthCookieResponse { expires_at }))
    }

    pub fn token_grpc(
        token: String,
        expires_at: i64,
    ) -> Result<Response<AuthTokenResponse>, tonic::Status> {
        Ok(Response::new(AuthTokenResponse { token, expires_at }))
    }
}

use crate::proto::common::{SingleResponse, StandardResponse};
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::HttpResponse;
use tonic::Response;

pub struct AuthController;

impl AuthController {
    pub fn cookie_http(token: &str, message: &str) -> HttpResponse {
        let cookie = Cookie::build("auth_token", token.to_string())
            .path("/")
            .http_only(true)
            .secure(false)
            .same_site(SameSite::Lax)
            .finish();

        HttpResponse::Ok()
            .cookie(cookie)
            .json(StandardResponse {
                message: message.to_string(),
            })
    }

    pub fn token_http(token: &str, message: &str) -> HttpResponse {
        HttpResponse::Ok().json(SingleResponse {
            message: message.to_string(),
            data: Some(token.to_string()),
        })
    }

    pub fn logout_http() -> HttpResponse {
        let cookie = Cookie::build("auth_token", "")
            .path("/")
            .http_only(true)
            .max_age(Duration::ZERO)
            .finish();

        HttpResponse::Ok()
            .cookie(cookie)
            .json(StandardResponse {
                message: "Logout successfully".to_string(),
            })
    }

    pub fn cookie_grpc(
        token: &str,
        ttl: u64,
        message: &str,
    ) -> Result<Response<StandardResponse>, tonic::Status> {
        let cookie_header = format!(
            "auth_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
            token, ttl
        );

        let mut response = Response::new(StandardResponse {
            message: message.to_string(),
        });
        response
            .metadata_mut()
            .insert(
                "set-cookie",
                cookie_header
                    .parse()
                    .map_err(|_| tonic::Status::internal("failed to set cookie metadata"))?,
            );
        Ok(response)
    }

    pub fn token_grpc(
        token: &str,
        message: &str,
    ) -> Result<Response<SingleResponse>, tonic::Status> {
        Ok(Response::new(SingleResponse {
            message: message.to_string(),
            data: Some(token.to_string()),
        }))
    }
}

pub fn validate_sign_in(email: &str, password: &str) -> Result<(), String> {
    let email = email.trim();
    let password = password.trim();

    if email.is_empty() || password.is_empty() {
        return Err("Email and password are required".to_string());
    }
    if email.len() < 6 || email.len() > 128 {
        return Err("Email must be between 6 and 128 characters".to_string());
    }
    if !email.contains('@') {
        return Err("Invalid email address".to_string());
    }
    if password.len() < 8 || password.len() > 2048 {
        return Err("Password must be between 8 and 2048 characters".to_string());
    }
    Ok(())
}

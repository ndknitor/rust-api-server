use crate::config::Config;
use crate::controllers::auth::AuthController;
use crate::libs::jwt::{build_claims, encode_token};
use crate::proto::auth::AuthRequest;
use actix_web::{HttpResponse, Responder, web};

fn issue_token(username: &str, cfg: &Config) -> Result<(String, i64), HttpResponse> {
    let claims = build_claims(
        username.to_string(),
        cfg.jwt_ttl,
        vec!["user".to_string()],
        vec!["read:users".to_string(), "read:orders".to_string()],
    );

    let token = encode_token(&claims, cfg.jwt_secret.as_str())
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    let expires_at = AuthController::expires_at(cfg.jwt_ttl);
    Ok((token, expires_at))
}

pub async fn login_cookie(payload: web::Json<AuthRequest>) -> impl Responder {
    let req = payload.into_inner();
    if req.username.trim().is_empty() || req.password.trim().is_empty() {
        return HttpResponse::BadRequest().finish();
    }

    let cfg = Config::from_env();
    match issue_token(&req.username, &cfg) {
        Ok((token, expires_at)) => AuthController::cookie_http(token, expires_at),
        Err(res) => res,
    }
}

pub async fn login_token(payload: web::Json<AuthRequest>) -> impl Responder {
    let req = payload.into_inner();
    if req.username.trim().is_empty() || req.password.trim().is_empty() {
        return HttpResponse::BadRequest().finish();
    }

    let cfg = Config::from_env();
    match issue_token(&req.username, &cfg) {
        Ok((token, expires_at)) => AuthController::token_http(token, expires_at),
        Err(res) => res,
    }
}

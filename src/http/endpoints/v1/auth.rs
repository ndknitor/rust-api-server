use crate::config::Config;
use crate::controllers::auth::{validate_sign_in, AuthController};
use crate::libs::jwt::{build_claims, encode_token, Claims};
use crate::proto::common::{SingleResponse, StandardResponse};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

// --- Request types (HTTP-specific, deserialized from JSON) ---

#[derive(Debug, Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct DebugQuery {
    pub role: Option<String>,
    pub user_policy: Option<String>,
}

// --- Helpers ---

fn issue_token(
    sub: &str,
    roles: Vec<String>,
    policies: Vec<String>,
    cfg: &Config,
) -> Result<String, HttpResponse> {
    let claims = build_claims(sub.to_string(), cfg.jwt_ttl, roles, policies);
    encode_token(&claims, &cfg.jwt_secret)
        .map_err(|_| HttpResponse::InternalServerError().finish())
}

fn has_bearer_header(req: &HttpRequest) -> bool {
    req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.starts_with("Bearer "))
        .unwrap_or(false)
}

fn get_claims(req: &HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

fn is_cookie_auth(req: &HttpRequest) -> bool {
    !has_bearer_header(req) && req.cookie("auth_token").is_some()
}

// --- Endpoints ---

/// GET /auth/debug/cookie/{id}?role=...&user_policy=...
/// Non-production only. Creates a cookie token with arbitrary claims.
pub async fn debug_cookie(
    req: HttpRequest,
    path: web::Path<i64>,
    query: web::Query<DebugQuery>,
) -> impl Responder {
    let cfg = Config::from_env();
    if cfg.environment == "production" {
        return HttpResponse::NotFound().finish();
    }
    if has_bearer_header(&req) {
        return HttpResponse::Forbidden().finish();
    }

    let id = path.into_inner();
    let role = query.role.clone().unwrap_or_else(|| "user".to_string());
    let policy = query.user_policy.clone().unwrap_or_default();
    let policies = if policy.is_empty() { vec![] } else { vec![policy] };

    match issue_token(&id.to_string(), vec![role], policies, &cfg) {
        Ok(token) => AuthController::cookie_http(&token, "Authenticate successfully"),
        Err(res) => res,
    }
}

/// GET /auth/debug/jwt/{id}?role=...&user_policy=...
/// Non-production only. Returns a JWT with arbitrary claims.
pub async fn debug_jwt(
    path: web::Path<i64>,
    query: web::Query<DebugQuery>,
) -> impl Responder {
    let cfg = Config::from_env();
    if cfg.environment == "production" {
        return HttpResponse::NotFound().finish();
    }

    let id = path.into_inner();
    let role = query.role.clone().unwrap_or_else(|| "user".to_string());
    let policy = query.user_policy.clone().unwrap_or_default();
    let policies = if policy.is_empty() { vec![] } else { vec![policy] };

    match issue_token(&id.to_string(), vec![role], policies, &cfg) {
        Ok(token) => AuthController::token_http(&token, "Authenticate successfully"),
        Err(res) => res,
    }
}

/// POST /auth/login/cookie
/// Authenticates with email/password and sets an auth cookie.
pub async fn login_cookie(
    req: HttpRequest,
    payload: web::Json<SignInRequest>,
) -> impl Responder {
    if has_bearer_header(&req) {
        return HttpResponse::BadRequest().json(StandardResponse {
            message: "Authorization headers are not allowed".to_string(),
        });
    }

    let body = payload.into_inner();
    if let Err(msg) = validate_sign_in(&body.email, &body.password) {
        return HttpResponse::BadRequest().json(StandardResponse { message: msg });
    }

    // TODO: verify credentials against database
    let cfg = Config::from_env();
    let roles = vec!["user".to_string()];
    let policies = vec!["read:seats".to_string()];

    match issue_token(&body.email, roles, policies, &cfg) {
        Ok(token) => AuthController::cookie_http(&token, "Authenticate successfully"),
        Err(res) => res,
    }
}

/// POST /auth/login/jwt
/// Authenticates with email/password and returns a JWT.
pub async fn login_jwt(payload: web::Json<SignInRequest>) -> impl Responder {
    let body = payload.into_inner();
    if let Err(msg) = validate_sign_in(&body.email, &body.password) {
        return HttpResponse::BadRequest().json(StandardResponse { message: msg });
    }

    // TODO: verify credentials against database
    let cfg = Config::from_env();
    let roles = vec!["user".to_string()];
    let policies = vec!["read:seats".to_string()];

    match issue_token(&body.email, roles, policies, &cfg) {
        Ok(token) => AuthController::token_http(&token, "Authenticate successfully"),
        Err(res) => res,
    }
}

/// GET /auth/logout
/// Clears the auth cookie. Only works for cookie-based auth.
pub async fn logout(req: HttpRequest) -> impl Responder {
    if !is_cookie_auth(&req) {
        return HttpResponse::Forbidden().json(StandardResponse {
            message: "Only cookie-based authentication can be logged out".to_string(),
        });
    }
    AuthController::logout_http()
}

/// GET /auth/refresh
/// Refreshes the current token (cookie or JWT).
pub async fn refresh(req: HttpRequest) -> impl Responder {
    let claims = match get_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let cfg = Config::from_env();
    match issue_token(&claims.sub, claims.roles, claims.policies, &cfg) {
        Ok(token) => {
            if is_cookie_auth(&req) {
                AuthController::cookie_http(&token, "Refresh successfully")
            } else {
                AuthController::token_http(&token, "Refresh successfully")
            }
        }
        Err(res) => res,
    }
}

/// GET /auth/authorize
/// Returns the current user's ID from the token claims.
pub async fn authorize(req: HttpRequest) -> impl Responder {
    match get_claims(&req) {
        Some(claims) => HttpResponse::Ok().json(claims.sub),
        None => HttpResponse::Unauthorized().finish(),
    }
}

/// POST /auth/test
/// Test endpoint that validates SignInRequest format.
pub async fn test(payload: web::Json<SignInRequest>) -> impl Responder {
    let body = payload.into_inner();
    if let Err(msg) = validate_sign_in(&body.email, &body.password) {
        return HttpResponse::BadRequest().json(StandardResponse { message: msg });
    }
    HttpResponse::Ok().finish()
}

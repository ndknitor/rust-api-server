use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::Response,
};

use crate::jwt::{Claims, decode_token};

fn extract_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get(header::AUTHORIZATION)
        && let Ok(auth_str) = auth.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        return Some(token.to_string());
    }

    if let Some(cookie_header) = headers.get(header::COOKIE)
        && let Ok(cookie_str) = cookie_header.to_str()
    {
        for part in cookie_str.split(';') {
            let cookie = part.trim();
            if let Some(token) = cookie.strip_prefix("auth_token=") {
                return Some(token.to_string());
            }
        }
    }

    None
}

#[derive(Clone)]
pub struct JwtAuth {
    secret: String,
    roles: Option<Vec<String>>,
    policies: Option<Vec<String>>,
}

impl JwtAuth {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            roles: None,
            policies: None,
        }
    }

    pub fn with_roles(secret: impl Into<String>, roles: Vec<&str>) -> Self {
        Self {
            secret: secret.into(),
            roles: Some(roles.into_iter().map(String::from).collect()),
            policies: None,
        }
    }

    pub fn with_policies(secret: impl Into<String>, policies: Vec<&str>) -> Self {
        Self {
            secret: secret.into(),
            roles: None,
            policies: Some(policies.into_iter().map(String::from).collect()),
        }
    }

    pub fn with_rules(secret: impl Into<String>, roles: Vec<&str>, policies: Vec<&str>) -> Self {
        Self {
            secret: secret.into(),
            roles: Some(roles.into_iter().map(String::from).collect()),
            policies: Some(policies.into_iter().map(String::from).collect()),
        }
    }
}

pub async fn jwt_authorize(
    State(auth): State<JwtAuth>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = extract_token(req.headers()).ok_or(StatusCode::UNAUTHORIZED)?;

    let claims =
        decode_token(&token, auth.secret.as_str()).map_err(|_| StatusCode::UNAUTHORIZED)?;

    if let Some(required_roles) = auth.roles {
        if !claims.roles.iter().any(|r| required_roles.contains(r)) {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    if let Some(required_policies) = auth.policies {
        if !required_policies
            .iter()
            .all(|p| claims.policies.contains(p))
        {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    req.extensions_mut().insert::<Claims>(claims);

    Ok(next.run(req).await)
}

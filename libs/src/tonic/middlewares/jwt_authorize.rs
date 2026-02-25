use tonic::metadata::MetadataMap;
use tonic::{Request, Status};

use crate::jwt::{Claims, decode_token};

fn extract_token(metadata: &MetadataMap) -> Option<String> {
    if let Some(auth) = metadata.get("authorization")
        && let Ok(auth_str) = auth.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        return Some(token.to_string());
    }

    if let Some(cookie_header) = metadata.get("cookie")
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
pub struct JwtAuthInterceptor {
    secret: String,
    roles: Option<Vec<String>>,
    policies: Option<Vec<String>>,
}

impl JwtAuthInterceptor {
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

    pub fn intercept(&self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let token = extract_token(req.metadata())
            .ok_or_else(|| Status::unauthenticated("missing token"))?;

        let claims = decode_token(&token, self.secret.as_str())
            .map_err(|_| Status::unauthenticated("invalid token"))?;

        if let Some(required_roles) = &self.roles
            && !claims.roles.iter().any(|r| required_roles.contains(r))
        {
            return Err(Status::permission_denied("insufficient role"));
        }

        if let Some(required_policies) = &self.policies
            && !required_policies
                .iter()
                .all(|p| claims.policies.contains(p))
        {
            return Err(Status::permission_denied("insufficient policy"));
        }

        req.extensions_mut().insert::<Claims>(claims);
        Ok(req)
    }
}

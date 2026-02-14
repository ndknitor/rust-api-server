use crate::libs::jwt::decode_token;
use ::tonic::{Request, Status, metadata::MetadataMap, service::Interceptor};

fn extract_token(metadata: &MetadataMap) -> Option<String> {
    let raw = metadata.get("authorization")?.to_str().ok()?;

    if raw.starts_with("Bearer ") {
        Some(raw.trim_start_matches("Bearer ").to_string())
    } else {
        None
    }
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

impl Interceptor for JwtAuth {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let token = extract_token(request.metadata())
            .ok_or_else(|| Status::unauthenticated("missing or invalid bearer token"))?;

        let claims = decode_token(&token, self.secret.as_str())
            .map_err(|_| Status::unauthenticated("invalid token"))?;

        if let Some(required_roles) = &self.roles {
            if !claims.roles.iter().any(|r| required_roles.contains(r)) {
                return Err(Status::permission_denied("missing required role"));
            }
        }

        if let Some(required_policies) = &self.policies {
            if !required_policies.iter().all(|p| claims.policies.contains(p)) {
                return Err(Status::permission_denied("missing required policy"));
            }
        }

        request.extensions_mut().insert(claims);
        Ok(request)
    }
}

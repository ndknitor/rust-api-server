use actix_web::HttpMessage;
use actix_web::{
    Error, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use crate::libs::jwt::decode_token;
use std::{
    rc::Rc,
    task::{Context, Poll},
};

fn extract_token(req: &ServiceRequest) -> Option<String> {
    if let Some(auth) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str.trim_start_matches("Bearer ").to_string());
            }
        }
    }

    if let Some(cookie) = req.cookie("auth_token") {
        return Some(cookie.value().to_string());
    }

    None
}

pub struct JwtAuth {
    secret: String,
    roles: Option<Vec<String>>,
    policies: Option<Vec<String>>,
}

impl JwtAuth {
    // Only authentication
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            roles: None,
            policies: None,
        }
    }

    // Auth + role requirement
    pub fn with_roles(secret: impl Into<String>, roles: Vec<&str>) -> Self {
        Self {
            secret: secret.into(),
            roles: Some(roles.into_iter().map(String::from).collect()),
            policies: None,
        }
    }

    // Auth + policy requirement
    pub fn with_policies(secret: impl Into<String>, policies: Vec<&str>) -> Self {
        Self {
            secret: secret.into(),
            roles: None,
            policies: Some(policies.into_iter().map(String::from).collect()),
        }
    }

    // Auth + both
    pub fn with_rules(secret: impl Into<String>, roles: Vec<&str>, policies: Vec<&str>) -> Self {
        Self {
            secret: secret.into(),
            roles: Some(roles.into_iter().map(String::from).collect()),
            policies: Some(policies.into_iter().map(String::from).collect()),
        }
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    secret: String,
    roles: Option<Vec<String>>,
    policies: Option<Vec<String>>,
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddleware {
            service: Rc::new(service),
            secret: self.secret.clone(),
            roles: self.roles.clone(),
            policies: self.policies.clone(),
        })
    }
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);
        let secret = self.secret.clone();
        let roles = self.roles.clone();
        let policies = self.policies.clone();

        Box::pin(async move {
            let token = match extract_token(&req) {
                Some(t) => t,
                None => {
                    return Ok(req.into_response(
                        HttpResponse::Unauthorized().finish().map_into_right_body()
                    ));
                }
            };

            let claims = match decode_token(&token, secret.as_str()) {
                Ok(c) => c,
                Err(_) => {
                    return Ok(req.into_response(
                        HttpResponse::Unauthorized().finish().map_into_right_body()
                    ));
                }
            };

            // Role rule: ANY match
            if let Some(required_roles) = roles {
                if !claims.roles.iter().any(|r| required_roles.contains(r)) {
                    return Ok(req.into_response(
                        HttpResponse::Forbidden().finish().map_into_right_body()
                    ));
                }
            }

            // Policy rule: ALL match
            if let Some(required_policies) = policies {
                if !required_policies.iter().all(|p| claims.policies.contains(p)) {
                    return Ok(req.into_response(
                        HttpResponse::Forbidden().finish().map_into_right_body()
                    ));
                }
            }

            req.extensions_mut().insert(claims);

            let res = srv.call(req).await?.map_into_left_body();
            Ok(res)
        })
    }
}

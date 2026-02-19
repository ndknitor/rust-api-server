use crate::controllers::auth::{AuthController, validate_sign_in};
use crate::inject::InjectFactory;
use crate::libs::jwt::Claims;
use crate::proto::auth::service_server::Service as GrpcAuthService;
use crate::proto::auth::{
    AuthorizeRequest, AuthorizeResponse, DebugAuthRequest, LogoutRequest, RefreshRequest,
    SignInRequest,
};
use crate::proto::common::{SingleResponse, StandardResponse};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct AuthEndpoint {
    factory: Arc<dyn InjectFactory>,
}

impl AuthEndpoint {
    pub fn new(factory: Arc<dyn InjectFactory>) -> Self {
        Self { factory }
    }

    fn issue_token(
        &self,
        sub: &str,
        roles: Vec<String>,
        policies: Vec<String>,
    ) -> Result<String, Status> {
        self.factory
            .auth_service()
            .issue_token(sub, roles, policies)
            .map_err(|e| Status::internal(e))
    }

    fn extract_claims<T>(&self, request: &Request<T>) -> Result<Claims, Status> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("missing or invalid bearer token"))?;

        self.factory
            .auth_service()
            .decode_token(token)
            .map_err(|_| Status::unauthenticated("invalid token"))
    }
}

#[tonic::async_trait]
impl GrpcAuthService for AuthEndpoint {
    async fn debug_cookie(
        &self,
        request: Request<DebugAuthRequest>,
    ) -> Result<Response<StandardResponse>, Status> {
        let auth = self.factory.auth_service();
        if auth.environment() == "production" {
            return Err(Status::not_found("not found"));
        }

        let req = request.into_inner();
        let role = if req.role.is_empty() {
            "user".to_string()
        } else {
            req.role
        };
        let policies = if req.user_policy.is_empty() {
            vec![]
        } else {
            vec![req.user_policy]
        };

        let token = self.issue_token(&req.id.to_string(), vec![role], policies)?;
        AuthController::cookie_grpc(&token, auth.jwt_ttl(), "Authenticate successfully")
    }

    async fn debug_jwt(
        &self,
        request: Request<DebugAuthRequest>,
    ) -> Result<Response<SingleResponse>, Status> {
        let auth = self.factory.auth_service();
        if auth.environment() == "production" {
            return Err(Status::not_found("not found"));
        }

        let req = request.into_inner();
        let role = if req.role.is_empty() {
            "user".to_string()
        } else {
            req.role
        };
        let policies = if req.user_policy.is_empty() {
            vec![]
        } else {
            vec![req.user_policy]
        };

        let token = self.issue_token(&req.id.to_string(), vec![role], policies)?;
        AuthController::token_grpc(&token, "Authenticate successfully")
    }

    async fn login_cookie(
        &self,
        request: Request<SignInRequest>,
    ) -> Result<Response<StandardResponse>, Status> {
        let req = request.into_inner();
        validate_sign_in(&req.email, &req.password)
            .map_err(|msg| Status::invalid_argument(msg))?;

        // TODO: verify credentials against database
        let roles = vec!["user".to_string()];
        let policies = vec!["read:seats".to_string()];

        let auth = self.factory.auth_service();
        let token = self.issue_token(&req.email, roles, policies)?;
        AuthController::cookie_grpc(&token, auth.jwt_ttl(), "Authenticate successfully")
    }

    async fn login_token(
        &self,
        request: Request<SignInRequest>,
    ) -> Result<Response<SingleResponse>, Status> {
        let req = request.into_inner();
        validate_sign_in(&req.email, &req.password)
            .map_err(|msg| Status::invalid_argument(msg))?;

        // TODO: verify credentials against database
        let roles = vec!["user".to_string()];
        let policies = vec!["read:seats".to_string()];

        let token = self.issue_token(&req.email, roles, policies)?;
        AuthController::token_grpc(&token, "Authenticate successfully")
    }

    async fn test(
        &self,
        request: Request<SignInRequest>,
    ) -> Result<Response<StandardResponse>, Status> {
        let req = request.into_inner();
        validate_sign_in(&req.email, &req.password)
            .map_err(|msg| Status::invalid_argument(msg))?;

        Ok(Response::new(StandardResponse {
            message: String::new(),
        }))
    }

    async fn logout(
        &self,
        _request: Request<LogoutRequest>,
    ) -> Result<Response<StandardResponse>, Status> {
        // gRPC uses Bearer tokens, not cookies — logout is not applicable
        Err(Status::permission_denied(
            "only cookie-based authentication can be logged out",
        ))
    }

    async fn refresh(
        &self,
        request: Request<RefreshRequest>,
    ) -> Result<Response<SingleResponse>, Status> {
        let claims = self.extract_claims(&request)?;
        let token = self.issue_token(&claims.sub, claims.roles, claims.policies)?;
        AuthController::token_grpc(&token, "Refresh successfully")
    }

    async fn authorize(
        &self,
        request: Request<AuthorizeRequest>,
    ) -> Result<Response<AuthorizeResponse>, Status> {
        let claims = self.extract_claims(&request)?;
        Ok(Response::new(AuthorizeResponse {
            user_id: claims.sub,
        }))
    }
}

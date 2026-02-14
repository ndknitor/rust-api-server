use crate::controllers::auth::AuthController;
use crate::libs::jwt::{build_claims, encode_token};
use crate::proto::auth::service_server::Service as GrpcAuthService;
use crate::proto::auth::{AuthCookieResponse, AuthRequest, AuthTokenResponse};
use tonic::{Request, Response, Status};

pub struct AuthEndpoint {
    jwt_secret: String,
    jwt_ttl: u64,
}

impl AuthEndpoint {
    pub fn new(jwt_secret: String, jwt_ttl: u64) -> Self {
        Self { jwt_secret, jwt_ttl }
    }

    fn issue_token(&self, username: &str) -> Result<(String, i64), Status> {
        let claims = build_claims(
            username.to_string(),
            self.jwt_ttl,
            vec!["user".to_string()],
            vec!["read:users".to_string(), "read:orders".to_string()],
        );

        let token = encode_token(&claims, self.jwt_secret.as_str())
            .map_err(|_| Status::internal("failed to create jwt token"))?;

        let expires_at = AuthController::expires_at(self.jwt_ttl);
        Ok((token, expires_at))
    }
}

#[tonic::async_trait]
impl GrpcAuthService for AuthEndpoint {
    async fn login_cookie(
        &self,
        request: Request<AuthRequest>,
    ) -> Result<Response<AuthCookieResponse>, Status> {
        let req = request.into_inner();
        if req.username.trim().is_empty() || req.password.trim().is_empty() {
            return Err(Status::invalid_argument("username and password are required"));
        }

        let (token, expires_at) = self.issue_token(&req.username)?;
        let mut response = AuthController::cookie_grpc(expires_at)?;

        // gRPC clients can read this metadata and store it as cookie for HTTP requests.
        let cookie = format!(
            "auth_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
            token, self.jwt_ttl
        );
        response
            .metadata_mut()
            .insert(
                "set-cookie",
                cookie
                    .parse()
                    .map_err(|_| Status::internal("failed to set cookie metadata"))?,
            );

        Ok(response)
    }

    async fn login_token(
        &self,
        request: Request<AuthRequest>,
    ) -> Result<Response<AuthTokenResponse>, Status> {
        let req = request.into_inner();
        if req.username.trim().is_empty() || req.password.trim().is_empty() {
            return Err(Status::invalid_argument("username and password are required"));
        }

        let (token, expires_at) = self.issue_token(&req.username)?;
        AuthController::token_grpc(token, expires_at)
    }
}

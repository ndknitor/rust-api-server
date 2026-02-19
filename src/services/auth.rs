use crate::libs::jwt;
use crate::libs::jwt::Claims;

pub trait AuthServiceTrait: Send + Sync {
    fn issue_token(
        &self,
        sub: &str,
        roles: Vec<String>,
        policies: Vec<String>,
    ) -> Result<String, String>;

    fn decode_token(&self, token: &str) -> Result<Claims, String>;

    fn jwt_ttl(&self) -> u64;

    fn environment(&self) -> &str;
}

pub struct AuthServiceImpl {
    jwt_secret: String,
    jwt_ttl: u64,
    environment: String,
}

impl AuthServiceImpl {
    pub fn new(jwt_secret: String, jwt_ttl: u64, environment: String) -> Self {
        Self {
            jwt_secret,
            jwt_ttl,
            environment,
        }
    }
}

impl AuthServiceTrait for AuthServiceImpl {
    fn issue_token(
        &self,
        sub: &str,
        roles: Vec<String>,
        policies: Vec<String>,
    ) -> Result<String, String> {
        let claims = jwt::build_claims(sub.to_string(), self.jwt_ttl, roles, policies);
        jwt::encode_token(&claims, &self.jwt_secret)
            .map_err(|e| format!("failed to create jwt token: {}", e))
    }

    fn decode_token(&self, token: &str) -> Result<Claims, String> {
        jwt::decode_token(token, &self.jwt_secret)
            .map_err(|e| format!("invalid token: {}", e))
    }

    fn jwt_ttl(&self) -> u64 {
        self.jwt_ttl
    }

    fn environment(&self) -> &str {
        &self.environment
    }
}

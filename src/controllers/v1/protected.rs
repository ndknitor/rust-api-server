use axum::Json;
use axum::extract::Extension;
use libs::jwt::Claims;
use serde_json::json;
use tonic::{Request, Response, Status};

use crate::pb;

pub async fn me(Extension(claims): Extension<Claims>) -> Json<serde_json::Value> {
    Json(json!({
        "sub": claims.sub,
        "roles": claims.roles,
        "policies": claims.policies,
    }))
}

#[derive(Default)]
pub struct ProtectedController;

#[tonic::async_trait]
impl pb::protected_service_server::ProtectedService for ProtectedController {
    async fn me(
        &self,
        request: Request<pb::ProtectedRequest>,
    ) -> Result<Response<pb::ProtectedResponse>, Status> {
        let claims = match request.extensions().get::<Claims>().cloned() {
            Some(claims) => claims,
            None => return Err(Status::unauthenticated("missing claims")),
        };

        Ok(Response::new(pb::ProtectedResponse {
            sub: claims.sub,
            roles: claims.roles,
            policies: claims.policies,
        }))
    }
}

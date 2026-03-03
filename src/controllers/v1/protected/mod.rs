pub mod me;

use tonic::{Request, Response, Status};

use libs::jwt::Claims;

use crate::pb;

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

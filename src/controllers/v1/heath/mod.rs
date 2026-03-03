pub mod healthz;

use tonic::{Request, Response, Status};

use crate::pb;

pub struct HeathController;

impl HeathController {
    pub fn new() -> Self {
        Self
    }
}

#[tonic::async_trait]
impl pb::heath_service_server::HeathService for HeathController {
    async fn check(
        &self,
        _request: Request<pb::HeathRequest>,
    ) -> Result<Response<pb::HeathResponse>, Status> {
        Ok(Response::new(healthz::heath_response()))
    }
}

use axum::{Json};
use tonic::{Request, Response, Status};

use crate::{pb};

pub struct HeathController {
}

impl HeathController {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl pb::heath_service_server::HeathService for HeathController {
    async fn check(
        &self,
        _request: Request<pb::HeathRequest>,
    ) -> Result<Response<pb::HeathResponse>, Status> {
        Ok(Response::new(heath_response()))
    }
}

pub async fn healthz() -> Json<pb::HeathResponse> {
    Json(heath_response())
}

fn heath_response() -> pb::HeathResponse {
    pb::HeathResponse {
        status: "ok".to_string(),
        service: "rust-api-server".to_string(),
    }
}

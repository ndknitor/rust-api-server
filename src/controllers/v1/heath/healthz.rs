use axum::Json;

use crate::pb;

pub async fn healthz() -> Json<pb::HeathResponse> {
    Json(heath_response())
}

pub fn heath_response() -> pb::HeathResponse {
    pb::HeathResponse {
        status: "ok".to_string(),
        service: "rust-api-server".to_string(),
    }
}

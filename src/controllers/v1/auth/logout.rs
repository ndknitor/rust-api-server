use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
    Json,
};
use tonic::{Request, Response, Status};

use crate::pb;

use super::{AuthControllerError, clear_auth_cookie, AuthState};

pub async fn logout() -> Result<impl IntoResponse, StatusCode> {
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_str("auth_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    headers.insert(SET_COOKIE, value);

    Ok((headers, Json(pb::LogoutResponse { status: "ok".to_string() })))
}

// gRPC handler
pub async fn grpc_logout(
    _service: &AuthState,
    _request: Request<pb::LogoutRequest>,
) -> Result<Response<pb::LogoutResponse>, Status> {
    let cookie_meta = tonic::metadata::MetadataValue::try_from(clear_auth_cookie().as_str())
        .map_err(|_| AuthControllerError::InvalidCookie)?;

    let mut response = Response::new(pb::LogoutResponse {
        status: "ok".to_string(),
    });
    response.metadata_mut().insert("set-cookie", cookie_meta);

    Ok(response)
}

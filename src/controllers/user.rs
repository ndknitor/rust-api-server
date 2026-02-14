use crate::proto::user::GetUsersResponse;
use actix_web::HttpResponse;
use tonic::Response;

pub struct UserController(pub GetUsersResponse);

impl UserController {
    pub fn from_users(users: Vec<String>) -> Self {
        Self(GetUsersResponse { users })
    }

    pub fn to_http(&self) -> HttpResponse {
        HttpResponse::Ok().json(&self.0)
    }

    pub fn to_grpc(self) -> Result<Response<GetUsersResponse>, tonic::Status> {
        Ok(Response::new(self.0))
    }
}

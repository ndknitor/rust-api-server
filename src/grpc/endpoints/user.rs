use crate::controllers::user::UserController;
use crate::proto::user::service_server::Service as GrpcUserService;
use crate::proto::user::{GetUsersRequest, GetUsersResponse};
use crate::services::UserService;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct UserEndpoint<S: UserService> {
    user_service: Arc<S>,
}

impl<S: UserService> UserEndpoint<S> {
    pub fn new(user_service: Arc<S>) -> Self {
        Self { user_service }
    }
}

#[tonic::async_trait]
impl<S: UserService + 'static> GrpcUserService for UserEndpoint<S> {
    async fn get_users(
        &self,
        _request: Request<GetUsersRequest>,
    ) -> Result<Response<GetUsersResponse>, Status> {
        let users = self.user_service.get_users().await;
        UserController::from_users(users).to_grpc()
    }
}

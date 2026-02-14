use crate::controllers::user::UserController;
use crate::services::UserService;
use actix_web::{Responder, web};
use std::sync::Arc;

pub async fn get_users(service: web::Data<Arc<dyn UserService>>) -> impl Responder {
    let users = service.get_users().await;
    UserController::from_users(users).to_http()
}

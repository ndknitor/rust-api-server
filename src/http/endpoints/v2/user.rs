use actix_web::{HttpResponse, Responder, web};
use serde::Serialize;

#[derive(Serialize)]
pub struct UserV2 {
    pub id: u32,
    pub name: String,
    pub email: String,
}

pub async fn get_users_v2() -> impl Responder {
    HttpResponse::Ok().json(vec![
        UserV2 {
            id: 1,
            name: "Alice V2".into(),
            email: "alice@example.com".into(),
        }
    ])
}
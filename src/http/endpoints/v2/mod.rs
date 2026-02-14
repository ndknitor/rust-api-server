pub mod order;
pub mod user;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("", web::get().to(user::get_users_v2))
    );
    cfg.service(
        web::scope("/orders")
            .route("/{user_id}", web::get().to(order::get_orders))
    );
}

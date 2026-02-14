pub mod auth;
pub mod order;
pub mod user;

use actix_web::web;
use crate::config::Config;
use crate::libs::actix::middlewares::jwt_authorize::JwtAuth;

pub fn routes(cfg: &mut web::ServiceConfig) {
    let jwt_secret = Config::from_env().jwt_secret;

    cfg.service(
        web::scope("/auth")
            .route("/cookie", web::post().to(auth::login_cookie))
            .route("/token", web::post().to(auth::login_token))
    );
    cfg.service(
        web::scope("/users")
            .wrap(JwtAuth::new(jwt_secret.clone()))
            .route("", web::get().to(user::get_users))
    );
    cfg.service(
        web::scope("/orders")
            .wrap(JwtAuth::new(jwt_secret))
            .route("/{user_id}", web::get().to(order::get_orders))
    );
}

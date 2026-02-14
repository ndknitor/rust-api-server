pub mod auth;
pub mod seat;

use actix_web::web;
use crate::config::Config;
use crate::libs::actix::middlewares::jwt_authorize::JwtAuth;

pub fn routes(cfg: &mut web::ServiceConfig) {
    let jwt_secret = Config::from_env().jwt_secret;

    cfg.service(
        web::scope("/auth")
            // Public routes
            .route("/debug/cookie/{id}", web::get().to(auth::debug_cookie))
            .route("/debug/jwt/{id}", web::get().to(auth::debug_jwt))
            .route("/login/cookie", web::post().to(auth::login_cookie))
            .route("/login/jwt", web::post().to(auth::login_jwt))
            .route("/test", web::post().to(auth::test))
            // Protected routes (nested scope with JWT middleware)
            .service(
                web::scope("")
                    .wrap(JwtAuth::new(jwt_secret.clone()))
                    .route("/logout", web::get().to(auth::logout))
                    .route("/refresh", web::get().to(auth::refresh))
                    .route("/authorize", web::get().to(auth::authorize))
            )
    );
    cfg.service(
        web::scope("/seats")
            .route("", web::get().to(seat::get_seats))
            .route("/range", web::get().to(seat::get_seats_by_range))
            .route("", web::post().to(seat::create_seats))
            .route("", web::put().to(seat::update_seats))
            .route("", web::delete().to(seat::delete_seats))
    );
}

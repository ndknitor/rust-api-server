use actix_web::web;
use super::endpoints::v1;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")        // Context Path
            .service(
                web::scope("/v1") // Version 1
                    .configure(v1::routes)
            )
    );
}

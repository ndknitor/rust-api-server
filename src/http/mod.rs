mod endpoints;
mod middlewares;
mod routes;

use crate::config::Config;
use crate::services::{OrderServiceFactory, OrderServiceTransient, UserService};
use actix_cors::Cors;
// use actix_files::Files;
use actix_web::{App, HttpServer, web};
// use middlewares::request_logger::RequestLogger;
use std::sync::Arc;

/// Start HTTP server
/// - user_service: Singleton (shared Arc across all requests)
/// - order_service_factory: Scoped (factory creates new instance per request)
/// - order_service_transient: Transient (function creates new instance every call)
pub async fn start<U, F>(
    user_service: Arc<U>,
    order_service_factory: Arc<F>,
    order_service_transient: OrderServiceTransient,
) -> std::io::Result<()>
where
    U: UserService + 'static,
    F: OrderServiceFactory + 'static,
{
    let cfg = Config::from_env();
    println!("Starting HTTP server on http://{}:{}", cfg.host, cfg.http_port);

    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .supports_credentials();
        for origin in &cfg.cors_origins {
            cors = cors.allowed_origin(origin);
        }
        App::new()
            // Singleton: same Arc shared across all requests
            .app_data(web::Data::<Arc<dyn UserService>>::new(user_service.clone()))
            // Scoped: factory registered, controller calls factory.create() once per request
            .app_data(web::Data::<Arc<dyn OrderServiceFactory>>::new(
                order_service_factory.clone(),
            ))
            // Transient: function pointer, controller calls it every time it needs an instance
            .app_data(web::Data::new(order_service_transient))
            // .wrap(RequestLogger)
            .wrap(cors)
            // Serve static file
            // .service(Files::new("/", "./wwwroot").index_file("index.html"))
            .configure(routes::config)
    })
    .bind((cfg.host.as_str(), cfg.http_port))?
    .run()
    .await
}

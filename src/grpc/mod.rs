mod endpoints;

use crate::config::Config;
use crate::proto;
use endpoints::auth::AuthEndpoint;
use endpoints::seat::SeatEndpoint;
use http::header::{HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use http::Method;
use sea_orm::DatabaseConnection;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer, Any};

pub async fn start(
    db: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg = Config::from_env();
    let addr = format!("{}:{}", cfg.host, cfg.grpc_port).parse()?;

    println!("Starting gRPC server on grpc://{}", addr);

    let cors = if cfg.cors_origins.is_empty() {
        CorsLayer::new().allow_origin(Any)
    } else {
        let origins = cfg
            .cors_origins
            .iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect::<Vec<_>>();
        CorsLayer::new().allow_origin(AllowOrigin::list(origins))
    }
    .allow_methods([Method::POST, Method::OPTIONS])
    .allow_headers([
        CONTENT_TYPE,
        AUTHORIZATION,
        HeaderName::from_static("x-grpc-web"),
        HeaderName::from_static("x-user-agent"),
        HeaderName::from_static("grpc-timeout"),
    ]);

    let auth_endpoint = AuthEndpoint::new(cfg.jwt_secret.clone(), cfg.jwt_ttl, cfg.environment.clone());
    let seat_endpoint = SeatEndpoint::new(db);

    Server::builder()
        .accept_http1(true)
        .layer(
            ServiceBuilder::new()
                .layer(GrpcWebLayer::new())
                .layer(cors),
        )
        .add_service(proto::auth::service_server::ServiceServer::new(auth_endpoint))
        .add_service(proto::seat::service_server::ServiceServer::new(seat_endpoint))
        .serve(addr)
        .await?;

    Ok(())
}

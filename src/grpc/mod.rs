mod endpoints;

use crate::config::Config;
use crate::libs::tonic::middlewares::jwt_authorize::JwtAuth;
use crate::proto;
use crate::services::{OrderServiceFactory, UserService};
use endpoints::auth::AuthEndpoint;
use endpoints::order::OrderEndpoint;
use endpoints::user::UserEndpoint;
use http::header::{HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use http::Method;
use std::sync::Arc;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer, Any};

/// Start gRPC server
/// - user_service: Singleton (shared Arc across all requests)
/// - order_service_factory: Scoped (factory creates new instance per request)
pub async fn start<U, F>(
    user_service: Arc<U>,
    order_service_factory: Arc<F>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    U: UserService + 'static,
    F: OrderServiceFactory + 'static,
{
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

    let user_endpoint = UserEndpoint::new(user_service);
    let order_endpoint = OrderEndpoint::new(order_service_factory);
    let auth_endpoint = AuthEndpoint::new(cfg.jwt_secret.clone(), cfg.jwt_ttl);
    let jwt_secret = cfg.jwt_secret.clone();

    Server::builder()
        .accept_http1(true)
        .layer(
            ServiceBuilder::new()
                .layer(GrpcWebLayer::new())
                .layer(cors),
        )
        .add_service(proto::auth::service_server::ServiceServer::new(auth_endpoint))
        .add_service(proto::user::service_server::ServiceServer::with_interceptor(
            user_endpoint,
            JwtAuth::new(jwt_secret.clone()),
        ))
        .add_service(proto::order::service_server::ServiceServer::with_interceptor(
            order_endpoint,
            JwtAuth::new(jwt_secret),
        ))
        .serve(addr)
        .await?;

    Ok(())
}

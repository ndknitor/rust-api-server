use axum::Router;
use axum::http::{HeaderValue, Method, header};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;

mod config;
mod controllers;
mod inject;
mod services;

use inject::{InjectFactory, InjectFactoryImpl};

pub mod pb {
    tonic::include_proto!("api");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let injector = InjectFactoryImpl::new();
    let config = injector.config()?;
    let jwt_service = injector.jwt_service()?;

    tracing_subscriber::fmt()
        .with_env_filter(config.rust_log.clone())
        .init();

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let cors_origin = config.cors_origin.clone();

    let http_app = Router::new().nest(
        "/api/v1",
        controllers::v1::http_router(config.clone(), jwt_service.clone()),
    );
    let grpc_routes = controllers::v1::grpc_router(config, jwt_service);

    let cors = build_cors(&cors_origin);
    let app = http_app.merge(grpc_routes).layer(cors);

    info!("HTTP + gRPC listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_cors(cors_origin: &str) -> CorsLayer {
    let origins: Vec<HeaderValue> = cors_origin
        .split(',')
        .filter_map(|s| HeaderValue::from_str(s.trim()).ok())
        .collect();

    let mut layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::COOKIE,
            header::HeaderName::from_static("x-grpc-web"),
            header::HeaderName::from_static("x-user-agent"),
            header::HeaderName::from_static("grpc-timeout"),
        ])
        .allow_credentials(true)
        .expose_headers([
            header::HeaderName::from_static("grpc-status"),
            header::HeaderName::from_static("grpc-message"),
            header::HeaderName::from_static("grpc-status-details-bin"),
            header::SET_COOKIE,
        ]);

    if !origins.is_empty() {
        layer = layer.allow_origin(origins);
    }

    layer
}

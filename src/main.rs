use axum::Router;
use axum::http::{HeaderValue, Method, header};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;

mod config;
mod controllers;
mod entities;
mod inject;
mod services;

use config::Config;
use inject::{InjectFactory, InjectFactoryImpl};

pub mod pb {
    tonic::include_proto!("api");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(config.rust_log.clone())
        .init();

    // Initialize injector with all singletons
    let injector = InjectFactoryImpl::init().await?;

    let config = injector.config()?;
    let jwt_service = injector.jwt_service()?;
    let db = injector.database()?;
    let auth_service: std::sync::Arc<dyn services::auth::Auth> = std::sync::Arc::new(services::auth::AuthImpl::new());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let cors_origin = config.cors_origin.clone();

    let http_app = Router::new().nest(
        "/api/v1",
        controllers::v1::http_router(config.clone(), jwt_service.clone(), auth_service.clone(), db.clone()),
    );
    let grpc_routes = controllers::v1::grpc_router(config, jwt_service, auth_service, db);

    let http_app = http_app.layer(build_http_cors(&cors_origin));
    let grpc_routes = grpc_routes.layer(build_grpc_cors(&cors_origin));
    let app = http_app.merge(grpc_routes);

    info!("HTTP + gRPC listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn parse_origins(cors_origin: &str) -> Vec<HeaderValue> {
    cors_origin
        .split(',')
        .filter_map(|s| HeaderValue::from_str(s.trim()).ok())
        .collect()
}

fn build_http_cors(cors_origin: &str) -> CorsLayer {
    let origins = parse_origins(cors_origin);

    let mut layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::COOKIE,
        ])
        .allow_credentials(true)
        .expose_headers([header::SET_COOKIE]);

    if !origins.is_empty() {
        layer = layer.allow_origin(origins);
    }

    layer
}

fn build_grpc_cors(cors_origin: &str) -> CorsLayer {
    let origins = parse_origins(cors_origin);

    let mut layer = CorsLayer::new()
        .allow_methods([Method::POST, Method::OPTIONS])
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
            header::SET_COOKIE,
            header::HeaderName::from_static("grpc-status"),
            header::HeaderName::from_static("grpc-message"),
            header::HeaderName::from_static("grpc-status-details-bin"),
        ]);

    if !origins.is_empty() {
        layer = layer.allow_origin(origins);
    }

    layer
}

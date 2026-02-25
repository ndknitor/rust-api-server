pub mod auth;
pub mod heath;
pub mod protected;

use std::sync::Arc;

use axum::{
    Extension, Router, middleware,
    routing::{get, post},
};
use libs::axum::middlewares::jwt_authorize::{JwtAuth, jwt_authorize};
use libs::tonic::middlewares::jwt_authorize::JwtAuthInterceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::service::{LayerExt, Routes};

use crate::{config::Config, pb, services::jwt::JWTService};

pub fn http_router(config: Arc<Config>, jwt_service: Arc<dyn JWTService>) -> Router {
    let jwt_auth = JwtAuth::with_policies(config.jwt_secret.clone(), vec!["protected.read"]);

    let protected_router = Router::new()
        .route("/protected", get(protected::me))
        .route_layer(middleware::from_fn_with_state(jwt_auth, jwt_authorize));

    let auth_router = Router::new()
        .route("/auth/jwt", post(auth::login_jwt_http))
        .route("/auth/cookie", post(auth::login_cookie_http))
        .route("/auth/logout", post(auth::logout_http))
        .layer(Extension(jwt_service));

    Router::new()
        .route("/", get(heath::healthz))
        .route("/healthz", get(heath::healthz))
        .merge(auth_router)
        .merge(protected_router)
        .with_state(config)
}

pub fn grpc_router(config: Arc<Config>, jwt_service: Arc<dyn JWTService>) -> Router {
    let jwt_secret = config.jwt_secret.clone();

    let heath = tonic_web::GrpcWebLayer::new().named_layer(
        pb::heath_service_server::HeathServiceServer::new(heath::HeathController::new()),
    );

    let auth = tonic_web::GrpcWebLayer::new().named_layer(
        pb::auth_service_server::AuthServiceServer::new(auth::AuthController::new(
            config,
            jwt_service,
        )),
    );

    let interceptor = JwtAuthInterceptor::with_policies(jwt_secret, vec!["protected.read"]);
    let protected_service =
        pb::protected_service_server::ProtectedServiceServer::new(protected::ProtectedController);
    let protected =
        InterceptedService::new(protected_service, move |req| interceptor.intercept(req));
    let protected = tonic_web::GrpcWebLayer::new().named_layer(protected);

    Routes::new(heath)
        .add_service(auth)
        .add_service(protected)
        .into_axum_router()
}

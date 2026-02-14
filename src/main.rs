mod config;
mod controllers;
mod grpc;
// mod http;
mod libs;
mod proto;
mod services;

use services::{OrderServiceFactoryImpl, UserServiceImpl};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    // Singleton: one instance shared across all requests
    let user_service = Arc::new(UserServiceImpl);

    // Scoped: factory creates new instance per request
    let order_service_factory = Arc::new(OrderServiceFactoryImpl);

    let grpc_user_service = user_service.clone();
    let grpc_order_factory = order_service_factory.clone();

    grpc::start(grpc_user_service, grpc_order_factory).await
}

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     env_logger::init();

//     // Singleton: one instance shared across all requests
//     let user_service = Arc::new(UserServiceImpl);

//     // Scoped: factory creates new instance per request
//     let order_service_factory = Arc::new(OrderServiceFactoryImpl);

//     // Transient: function creates new instance every call
//     let order_service_transient = create_order_service;

//     let grpc_user_service = user_service.clone();
//     let grpc_order_factory = order_service_factory.clone();

//     let grpc_task = tokio::spawn(async move {
//         grpc::start(grpc_user_service, grpc_order_factory).await
//     });

//     let http_result =
//         http::start(user_service, order_service_factory, order_service_transient).await;

//     match grpc_task.await {
//         Ok(Ok(())) => {}
//         Ok(Err(err)) => eprintln!("gRPC server failed: {err}"),
//         Err(err) => eprintln!("gRPC server task failed: {err}"),
//     }

//     http_result
// }

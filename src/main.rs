mod config;
mod controllers;
mod entities;
mod grpc;
// mod http;
mod libs;
mod proto;
mod services;

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let cfg = Config::from_env();

    let db = sea_orm::Database::connect(&cfg.database_url).await?;
    println!("Connected to database");

    grpc::start(db).await
}

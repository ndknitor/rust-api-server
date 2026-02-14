use std::env;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub http_port: u16,
    pub grpc_port: u16,
    pub jwt_secret: String,
    pub jwt_ttl: u64,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());

        let http_port = env::var("HTTP_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);

        let grpc_port = env::var("GRPC_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(50051);

        let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");

        let jwt_ttl = env::var("JWT_TTL")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(3600);

        let cors_origins = env::var("CORS_ORIGIN")
            .unwrap_or_default()
            .split(',')
            .map(|o| o.trim().to_string())
            .filter(|o| !o.is_empty())
            .collect::<Vec<_>>();

        Self {
            host,
            http_port,
            grpc_port,
            jwt_secret,
            jwt_ttl,
            cors_origins,
        }
    }
}

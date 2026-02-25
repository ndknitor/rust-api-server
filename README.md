# rust-api-server

Single binary Rust server using Axum + Tonic on one port.

## What this project does

- Serves **HTTP** and **gRPC** from the same socket (`HOST:PORT`)
- Supports **grpc-web** for browser clients
- Uses protobuf schema as shared response model for HTTP and gRPC
- Loads runtime config from environment via `Config`
- Uses a simple singleton injector (`InjectFactoryImpl`) for config

## Current HTTP and gRPC surface

- HTTP base path: `/api/v1`
- HTTP endpoints:
  - `GET /api/v1/`
  - `GET /api/v1/healthz`
- gRPC service:
  - `api.HeathService/Check`

Schema is defined in `proto/heath.proto`.

## Tech stack

- `axum` (HTTP router/server)
- `tonic` + `tonic-prost` (gRPC + generated code)
- `tonic-web` (grpc-web compatibility)
- `tokio` (runtime)
- `tracing` + `tracing-subscriber` (logging)

## Project layout

```txt
src/
  main.rs
  config.rs
  inject.rs
  controllers/
    mod.rs
    v1/
      mod.rs
      heath.rs
proto/
  heath.proto
build.rs

libs/
  Cargo.toml
  src/
    lib.rs
    jwt.rs
    actix/middlewares/jwt_authorize.rs
    axum/middlewares/jwt_authorize.rs
    tonic/middlewares/jwt_authorize.rs
```

## Configuration

Environment variables (`.env.example`):

```env
HOST=0.0.0.0
PORT=8080
JWT_SECRET=your-secret-key
JWT_TTL=3600
CORS_ORIGIN=http://localhost:3000,http://localhost:5173
```

`Config` is built from env in `src/config.rs`.

## Config injection pattern

- `InjectFactory` trait defines dependency accessors
- `InjectFactoryImpl` stores config in `OnceLock<Arc<Config>>`
- `config.clone()` in `main.rs` clones `Arc`, not `Config` data
- HTTP and gRPC handlers receive the same shared config instance

## Build and run

```bash
# build
cargo build

# run
cargo run

# check
cargo check
```

Server listens on `HOST:PORT` (default `0.0.0.0:8080`).

## HTTP usage example

```bash
curl http://localhost:8080/api/v1/healthz
```

Expected JSON (protobuf-backed):

```json
{
  "status": "ok",
  "service": "rust-api-server:8080@0.0.0.0"
}
```

## gRPC usage example

```bash
grpcurl -plaintext \
  -d '{}' \
  localhost:8080 api.HeathService/Check
```

## grpc-web support

gRPC routes are wrapped with `tonic_web::GrpcWebLayer`, so grpc-web clients can call the same service on the same port.

## Shared schema between HTTP and gRPC

`build.rs` generates Rust types from `proto/heath.proto` and adds serde derives.

The HTTP health handler returns `Json<pb::HeathResponse>` and the gRPC method returns `pb::HeathResponse`, so both transports use the same schema.

## libs crate (auth middleware)

The `libs` crate contains JWT helpers and middleware implementations:

- `libs::axum::middlewares::jwt_authorize`
- `libs::tonic::middlewares::jwt_authorize`
- `libs::actix::middlewares::jwt_authorize` (legacy compatibility)

JWT claims model:

```json
{
  "sub": "user-id",
  "exp": 1234567890,
  "roles": ["admin"],
  "policies": ["order.read"]
}
```

Authorization behavior across middleware:

- Missing/invalid token => unauthorized
- Role rule => ANY required role must match
- Policy rule => ALL required policies must match

## Docker

A production Dockerfile is included and exposes `8080`.

```bash
docker build -t rust-api-server .
docker run --rm -p 8080:8080 rust-api-server
```

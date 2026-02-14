# Rust Actix API

A Rust web API demonstrating dependency injection patterns with Actix-web (HTTP) and Tonic (gRPC).

## Features

- HTTP REST API with Actix-web
- gRPC API with Tonic
- Dependency injection with three lifetime patterns:
  - **Singleton**: Single instance shared across all requests
  - **Scoped**: New instance per request
  - **Transient**: New instance every time it's needed
- JWT authentication middleware
- CORS support
- Request logging

## Project Structure

```
src/
├── config.rs                 # Shared configuration (env-based)
├── main.rs                   # Application entry point
├── services/                 # Business logic layer
│   ├── mod.rs
│   ├── user.rs               # UserService (Singleton)
│   └── order.rs              # OrderService (Scoped + Transient)
├── http/                     # HTTP server (Actix-web)
│   ├── mod.rs                # Server setup
│   ├── routes.rs             # Route configuration
│   ├── controllers/
│   │   ├── v1/               # API v1 (Scoped OrderService)
│   │   │   ├── user.rs
│   │   │   └── order.rs
│   │   └── v2/               # API v2 (Transient OrderService)
│   │       ├── user.rs
│   │       └── order.rs
│   └── middlewares/
│       ├── jwt_authorize.rs  # JWT authentication
│       └── request_logger.rs # Request logging
├── grpc/                     # gRPC server (Tonic)
│   ├── mod.rs                # Server setup
│   └── controllers/
│       ├── user.rs
│       └── order.rs
└── proto/                    # Protocol Buffers
    ├── user.proto
    └── order.proto
```

## Dependency Injection Patterns

### Singleton
One instance shared across all requests. Use for stateless services, caches, connection pools.

```rust
// Registration
let user_service = Arc::new(UserServiceImpl);
.app_data(web::Data::<Arc<dyn UserService>>::new(user_service.clone()))

// Controller
pub async fn get_users(service: web::Data<Arc<dyn UserService>>) -> impl Responder {
    let users = service.get_users().await;  // Same instance every request
    HttpResponse::Ok().json(users)
}
```

### Scoped
New instance per request. Use for per-request state, database transactions.

```rust
// Registration
let order_factory = Arc::new(OrderServiceFactoryImpl);
.app_data(web::Data::<Arc<dyn OrderServiceFactory>>::new(order_factory.clone()))

// Controller
pub async fn get_orders(factory: web::Data<Arc<dyn OrderServiceFactory>>) -> impl Responder {
    let service = factory.create();  // New instance for this request
    let orders = service.get_orders(&user_id).await;
    HttpResponse::Ok().json(orders)
}
```

### Transient
New instance every time it's requested. Use when each operation needs isolated state.

```rust
// Registration
let order_transient = create_order_service;  // fn() -> Box<dyn OrderService>
.app_data(web::Data::new(order_transient))

// Controller
pub async fn get_orders(create_fn: web::Data<OrderServiceTransient>) -> impl Responder {
    let service1 = create_fn();  // New instance
    let service2 = create_fn();  // Another new instance (different from service1)
    // ...
}
```

## Environment Variables

Create a `.env` file based on `.env.example`:

```env
HOST=127.0.0.1
HTTP_PORT=8080
GRPC_PORT=50051
JWT_SECRET=your-secret-key
JWT_TTL=3600
CORS_ORIGIN=http://localhost:3000,http://localhost:5173
```

## Build & Run

```bash
# Build
cargo build

# Run (HTTP server on configured port)
cargo run

# Run tests
cargo test

# Check for errors
cargo check

# Lint
cargo clippy

# Format
cargo fmt
```

## API Endpoints

### HTTP (Actix-web)

| Method | Endpoint | Auth | DI Pattern | Description |
|--------|----------|------|------------|-------------|
| GET | `/api/v1/users` | JWT | Singleton | Get all users |
| GET | `/api/v1/orders/{user_id}` | JWT | Scoped | Get orders by user |
| GET | `/api/v2/users` | - | Singleton | Get all users (v2) |
| GET | `/api/v2/orders/{user_id}` | - | Transient | Get orders by user |

### gRPC (Tonic)

| Service | Method | DI Pattern |
|---------|--------|------------|
| UserService | GetUsers | Singleton |
| OrderService | GetOrders | Scoped |

## JWT Authentication

Protected endpoints require a valid JWT token:

```bash
# Header
Authorization: Bearer <token>

# Or Cookie
auth_token=<token>
```

JWT payload structure:
```json
{
  "sub": "user-id",
  "exp": 1234567890,
  "roles": ["admin", "user"],
  "policies": ["read", "write"]
}
```

### Role-based Authorization

```rust
// Require authentication only
.wrap(JwtAuth::new())

// Require specific roles (ANY match)
.wrap(JwtAuth::with_roles(vec!["admin", "moderator"]))

// Require specific policies (ALL must match)
.wrap(JwtAuth::with_policies(vec!["read", "write"]))

// Require both roles and policies
.wrap(JwtAuth::with_rules(vec!["admin"], vec!["delete"]))
```

## Running Both Servers

To run HTTP and gRPC servers concurrently, update `src/main.rs`:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let user_service = Arc::new(UserServiceImpl);
    let order_factory = Arc::new(OrderServiceFactoryImpl);
    let order_transient = create_order_service;

    tokio::try_join!(
        async {
            http::start(
                user_service.clone(),
                order_factory.clone(),
                order_transient
            ).await.map_err(|e| e.into())
        },
        grpc::start(user_service.clone(), order_factory.clone()),
    )?;

    Ok(())
}
```

## Database Migrations with SeaORM

SeaORM provides a CLI tool (`sea-orm-cli`) for managing database schema migrations in both directions: writing migrations in code and applying them to the database, or generating entity code from an existing database.

### Setup

Install the CLI tool:

```bash
cargo install sea-orm-cli
```

### Code to Database (Migration-first)

Write migrations in Rust and apply them to your database.

**1. Initialize the migration directory:**

```bash
sea-orm-cli migrate init
```

This creates a `migration/` directory with a sample migration.

**2. Create a new migration:**

```bash
sea-orm-cli migrate generate create_users_table
```

This generates a timestamped migration file in `migration/src/`. Edit it to define your schema:

**3. Apply migrations:**

```bash
# Apply all pending migrations
sea-orm-cli migrate up -u "postgres://user:pass@localhost/dbname"

# Apply N pending migrations
sea-orm-cli migrate up -n 1 -u "postgres://user:pass@localhost/dbname"

# Rollback last migration
sea-orm-cli migrate down -u "postgres://user:pass@localhost/dbname"

# Rollback N migrations
sea-orm-cli migrate down -n 2 -u "postgres://user:pass@localhost/dbname"

# Check migration status
sea-orm-cli migrate status -u "postgres://user:pass@localhost/dbname"

# Reset (rollback all, then re-apply)
sea-orm-cli migrate fresh -u "postgres://user:pass@localhost/dbname"

# Rollback all migrations
sea-orm-cli migrate reset -u "postgres://user:pass@localhost/dbname"
```

You can also set `DATABASE_URL` in your `.env` file to avoid passing `-u` every time.

**4. Generate entities from the migrated database:**

```bash
sea-orm-cli generate entity -o src/entities -u "postgres://user:pass@localhost/dbname"
```

### Database to Code (Database-first)

Generate Rust entity code from an existing database schema.

```bash
# Generate all entities
sea-orm-cli generate entity -o src/entities -u "postgres://user:pass@localhost/dbname"

# Generate specific tables only
sea-orm-cli generate entity -o src/entities -t users,orders -u "postgres://user:pass@localhost/dbname"

# Generate with serde derives for JSON serialization
sea-orm-cli generate entity -o src/entities --with-serde both -u "postgres://user:pass@localhost/dbname"

# Expand enum variants (useful for Postgres enums)
sea-orm-cli generate entity -o src/entities --expanded-format -u "postgres://user:pass@localhost/dbname"
```

This generates one file per table (e.g., `src/entities/users.rs`) containing the `Model`, `ActiveModel`, `Column`, and `Relation` definitions.

### Typical Workflow

| Approach | When to use |
|----------|-------------|
| **Code to DB** (migration-first) | Greenfield projects, version-controlled schema changes |
| **DB to Code** (database-first) | Existing databases, DBA-managed schemas |

A common hybrid workflow:

1. Write and apply migrations (code to DB)
2. Regenerate entities after each migration (DB to code)
3. Commit both migrations and generated entities to version control

## Reducing Binary Size

If you only need HTTP or gRPC, remove the unused module to reduce binary size.

### Remove gRPC (keep HTTP only)

1. Delete gRPC files and dependencies:

```bash
rm -rf src/grpc/
rm -rf proto/
rm build.rs
```

2. Update `Cargo.toml` - remove gRPC dependencies:

```toml
# Remove these lines:
tonic = "0.12"
prost = "0.13"

[build-dependencies]
tonic-build = "0.12"
```

3. Update `src/main.rs` - remove gRPC module:

```rust
mod config;
// mod grpc;  // Remove this line
mod http;
mod services;
```

4. Update `src/config.rs` - remove `grpc_port` field (optional).

### Disable Actix HTTP (keep gRPC only)

1. Update `src/main.rs` to use Tokio and call only gRPC:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let user_service = Arc::new(UserServiceImpl);
    let order_factory = Arc::new(OrderServiceFactoryImpl);

    grpc::start(user_service, order_factory).await
}
```

2. Comment out the HTTP module and imports:

```rust
mod config;
mod grpc;
// mod http;
mod services;
```

3. Optional cleanup (if you want to remove HTTP entirely):

```bash
rm -rf src/http/
```

4. Optional cleanup: remove HTTP dependencies from `Cargo.toml`:

```toml
# Remove these lines:
actix-web = "4"
actix-cors = "0.7"
```

5. Optional cleanup: update `src/config.rs` to remove `http_port`, `cors_origins`.

### Additional Binary Optimizations

Add to `Cargo.toml` for smaller release builds:

```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
panic = "abort"     # Abort on panic (no unwinding)
strip = true        # Strip symbols
```

Build with:

```bash
cargo build --release
```

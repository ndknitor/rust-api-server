# ---------- BUILD STAGE ----------
FROM rust:latest AS builder

WORKDIR /app

# Build tools: musl for static binary + protoc for tonic/prost codegen
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    protobuf-compiler \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Cache dependency compilation
COPY Cargo.toml Cargo.lock build.rs ./
COPY proto ./proto
COPY libs/Cargo.toml libs/Cargo.lock ./libs/
RUN mkdir -p src libs/src \
    && echo "fn main() {}" > src/main.rs \
    && echo "pub mod jwt;" > libs/src/lib.rs \
    && echo "pub struct Claims;" > libs/src/jwt.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm -rf ./src/ ./libs/src/

# Copy real sources and build final binary
COPY ./src/ ./src/
COPY ./libs/src ./libs/src
RUN rm -rf ./target/ && cargo build --release --target x86_64-unknown-linux-musl

# ---------- RUNTIME STAGE (ALPINE - OPTIONAL) ----------
FROM alpine:3.22
WORKDIR /app
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rust-api-server /app/main
USER 1000:1000
EXPOSE 8080
CMD ["/app/main"]

# ---------- RUNTIME STAGE (SCRATCH) ----------
# FROM scratch
# WORKDIR /app
# COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rust-api-server /app/main
# COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
# USER 1000:1000
# EXPOSE 8080
# ENTRYPOINT ["/app/main"]
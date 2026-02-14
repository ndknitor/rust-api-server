# ---------- BUILD STAGE ----------
FROM rust:latest AS builder

WORKDIR /app

# Install musl target + tools
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools ca-certificates

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm -rf src

# Copy real source
COPY src ./src

# Force rebuild of the binary (not just dependencies)
RUN touch src/main.rs && cargo build --release --target x86_64-unknown-linux-musl

# # ---------- RUNTIME STAGE FROM ALPINE ----------
# FROM alpine
# WORKDIR /app
# COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rust-api-server /app/main
# # COPY wwwroot /app/wwwroot
# USER nobody
# EXPOSE 3000
# ENV HOST=0.0.0.0
# ENV PORT=3000
# ENTRYPOINT ["/app/main"]

# ---------- RUNTIME STAGE FROM SCRATCH ----------
FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rust-api-server /app/main
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
# COPY wwwroot /app/wwwroot
USER 1000:1000
EXPOSE 8080
ENV HOST=0.0.0.0
ENV HTTP_PORT=8080
ENTRYPOINT ["/app/main"]
# ---- Build stage ----
FROM rust:1.93-bookworm AS builder

WORKDIR /src

# Copy workspace manifests and source code
COPY Cargo.toml Cargo.toml
COPY backend/ backend/
COPY common/ common/

# Create dummy game crate to satisfy workspace
COPY game/Cargo.toml game/Cargo.toml
RUN mkdir -p game/src && \
    echo "" > game/src/lib.rs && \
    echo "fn main() {}" > game/src/main.rs

ENV SQLX_OFFLINE=true
RUN cargo build --release -p backend

# ---- Runtime stage ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/backend /backend
COPY --from=builder /src/backend/migrations /migrations

EXPOSE 3000

CMD ["/backend"]

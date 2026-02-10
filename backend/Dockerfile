# ---- Build stage ----
FROM rust:1.93-bookworm AS builder

WORKDIR /src

# Copy workspace manifests first for dependency caching
COPY Cargo.toml Cargo.toml
COPY backend/Cargo.toml backend/Cargo.toml
COPY common/Cargo.toml common/Cargo.toml
COPY game/Cargo.toml game/Cargo.toml

# Create dummy source files so cargo can fetch and compile dependencies.
# This is a Docker layer caching optimization: Cargo.toml files change rarely
# compared to source code, so the expensive dependency compilation layer stays
# cached across most builds. Only the final rebuild of the actual crate code
# runs when source files change.
RUN mkdir -p backend/src common/src game/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "" > common/src/lib.rs && \
    echo "" > game/src/lib.rs && \
    echo "fn main() {}" > game/src/main.rs

ENV SQLX_OFFLINE=true
RUN cargo build --release -p backend


# Remove dummy source files to avoid confusion in the final build
# but keep the dummy game to prevent the workspace from thinking 
# the game crate is missing (since it has a different target)
RUN rm -rf backend/src common/src

# Copy real source code and rebuild
COPY common/ common/
COPY backend/ backend/

RUN cargo build --release -p backend

# ---- Runtime stage ----
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /src/target/release/backend /backend
COPY --from=builder /src/backend/migrations /migrations

EXPOSE 3000

ENTRYPOINT ["/backend"]

# ---- WASM build stage ----
FROM rust:1.93-bookworm AS wasm-builder

RUN cargo install wasm-pack

WORKDIR /src

# Copy workspace manifests for dependency caching
COPY Cargo.toml Cargo.toml
COPY backend/Cargo.toml backend/Cargo.toml
COPY common/Cargo.toml common/Cargo.toml
COPY game/Cargo.toml game/Cargo.toml

# Create dummy source files
RUN mkdir -p backend/src common/src game/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "" > common/src/lib.rs && \
    echo "" > game/src/lib.rs && \
    echo "fn main() {}" > game/src/main.rs

ENV SQLX_OFFLINE=true
RUN cargo build --release -p game 2>/dev/null || true

# Remove dummy source files to avoid confusion in the final build
# but keep the dummy backend to prevent the workspace from thinking 
# the backend crate is missing (since it has a different target)
RUN rm -rf common/src game/src

# Copy real source code and build script
COPY common/ common/
COPY game/ game/
COPY scripts/build-wasm.sh scripts/build-wasm.sh

RUN scripts/build-wasm.sh /wasm-out

# ---- Frontend build stage ----
FROM node:22-alpine AS frontend-builder

WORKDIR /app

COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

COPY frontend/ .
COPY --from=wasm-builder /wasm-out src/wasm/

RUN npm run build

# ---- Runtime stage ----
FROM nginx:alpine

COPY --from=frontend-builder /app/dist /usr/share/nginx/html
COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

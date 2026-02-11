# ---- WASM build stage ----
FROM rust:1.93-bookworm AS wasm-builder

RUN cargo install wasm-pack

WORKDIR /src

# Copy workspace manifests and source code
COPY Cargo.toml Cargo.toml
COPY common/ common/
COPY game/ game/
COPY scripts/build-wasm.sh scripts/build-wasm.sh

# Create dummy backend crate to satisfy workspace
COPY backend/Cargo.toml backend/Cargo.toml
RUN mkdir -p backend/src && \
    echo "fn main() {}" > backend/src/main.rs

ENV SQLX_OFFLINE=true
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

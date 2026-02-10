# rsdice — Backend

The backend server for rsdice, providing a REST API and WebSocket server for real-time game communication.

## Tech Stack

- **Rust 2024** with **Axum 0.8** (HTTP framework, WebSocket support)
- **SQLx** with **PostgreSQL 18** (compile-time checked queries)
- **JWT** (HttpOnly cookies) + **Argon2** password hashing for authentication
- **Tokio** async runtime
- **Tracing** for structured logging

## Prerequisites

- **Rust** (edition 2024) — install via [rustup](https://rustup.rs/)
- **Docker & Docker Compose** — for the PostgreSQL database
- **SQLx CLI** — `cargo install sqlx-cli --features postgres`

## Setup

### 1. Start PostgreSQL

```sh
docker compose -f compose.yml up -d
```

The container exposes PostgreSQL on port **5432** with credentials `rsdice` / `rsdice` and database `rsdice`.

### 2. Run migrations

```sh
sqlx migrate run
```

Migrations live in the `migrations/` directory.

### 3. Environment variables

Create a `.env` file in this directory:

```env
PORT=3000
JWT_SECRET=your-secret-key
DATABASE_URL=postgres://rsdice:rsdice@localhost:5432/rsdice
```

### 4. Run

```sh
cargo run -p backend
```

The server will start on `0.0.0.0:{PORT}`.

## Project Structure

```
backend/
├── compose.yml              # Docker Compose for PostgreSQL
├── migrations/              # SQLx database migrations
└── src/
    ├── main.rs              # Entry point — config, DB, router setup
    ├── models/              # Domain models (e.g. User)
    ├── prelude/             # Shared imports: AppState, Config, Error, Claims
    ├── repositories/        # Database access (repository pattern)
    └── routes/              # Axum route handlers and routers
```

## Key Patterns

- **Prelude** — All modules import `use crate::prelude::*;` for common types.
- **Error handling** — Per-domain error enums with `thiserror`, custom `Result<T>` aliases, `IntoResponse` implementations.
- **Router** — Each route module exposes `pub fn router(state: AppState) -> Router`, nested via `.nest()`.
- **Repository** — Structs borrowing `&PgPool`, using `sqlx::query!` / `sqlx::query_as!` with raw SQL.

## Database

The PostgreSQL container must be running for local builds (SQLx verifies queries at compile time).

After changing any database queries, regenerate offline metadata for CI:

```sh
cargo sqlx prepare -p backend
```

This allows CI to build with `SQLX_OFFLINE=true` without a live database.

## Testing

```sh
cargo test -p backend
```

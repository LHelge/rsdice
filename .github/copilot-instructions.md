# Copilot Instructions — rsdice

## Project Overview

rsdice is a turn-based online multiplayer dice game. Players compete on a map built from hexagonal tiles, each tile belonging to an **area**. The game follows Risk-like combat mechanics with dice rolling and area control.

### Game Rules

1. **Setup** — A map is generated from hexagonal tiles grouped into areas. Each player is randomly assigned a set of areas, and each owned area receives a random number of dice (1–8).
2. **Turns** — Players take turns in order. On a turn a player may attack any adjacent enemy area from one of their own areas.
3. **Combat** — The attacker rolls the number of dice on their attacking area; the defender rolls the dice on the defending area. If the attacker's total is **strictly greater**, the attacker captures the area and moves all dice except one from the attacking area to the captured area. On a **draw or defender higher**, the attacker loses all dice on the attacking area except one.
4. **End of turn** — When the player ends their turn (or has no valid attacks), they receive bonus dice equal to the size of their **largest group of connected areas**. These bonus dice are distributed randomly across all of their areas (each area is capped at 8 dice). Excess dice that cannot be placed are **stored** (up to a maximum of 60) and carried over for random allocation on subsequent turns.
5. **Elimination** — A player is eliminated when they lose all their areas. The last player standing wins.

---

## Tech Stack & Architecture

| Layer | Technology | Notes |
|---|---|---|
| **Backend** | Rust 2024, Axum 0.8, SQLx (PostgreSQL), JWT + Argon2 auth | REST API + WebSocket server |
| **Game client** | Rust 2024, Bevy 0.18 | Compiled to WebAssembly via `wasm-pack` |
| **Common** | Rust 2024 crate | Shared domain types used by backend and game |
| **Frontend** | Node, Vite, TypeScript, React, Tailwind CSS | Hosts the WASM game client |
| **Communication** | WebSockets | Between game client (Bevy/WASM) and backend (Axum) |
| **Database** | PostgreSQL 18 | Managed via Docker Compose, SQLx migrations |

### Workspace Structure

```
rsdice/                 # Cargo workspace (resolver = "3")
├── backend/            # Axum REST + WebSocket server
├── common/             # Shared game domain types
├── game/               # Bevy game client (compiles to WASM)
└── frontend/           # Vite + TypeScript + React (hosts WASM build)
```

---

## Code Conventions

### Rust General

- **Edition**: 2024 for all crates.
- **Workspace resolver**: 3.
- **Module re-exports**: Each `mod.rs` uses `mod foo; pub use foo::*;` to flatten the module tree.
- **Dependencies**: Always use `cargo add <crate>` to add new dependencies so versions stay current.

### Backend Patterns

- **Prelude** — The backend has a `prelude` module. All backend modules import it as `use crate::prelude::*;`.
- **Error handling** — Define per-domain error enums using `#[derive(Debug, thiserror::Error)]`. Create a custom `type Result<T> = std::result::Result<T, Error>;` alias in each error module. Implement `IntoResponse` to map errors to HTTP status codes.
- **Router structure** — Each route module exposes a `pub fn router(state: AppState) -> Router` function. The top-level router uses `.nest("/path", module::router(state))` for grouping.
- **Handlers** — Use Axum extractors (`State`, `Path`, `Json`, `Claims`). Document handlers with `///` doc comments.
- **Repository pattern** — Repository structs borrow `&PgPool`. Use `sqlx::query!` and `sqlx::query_as!` macros with raw SQL strings (`r#"..."#`). Return the crate's `Result<T>` type.
- **Auth** — JWT stored in HttpOnly cookies. `Claims` is implemented as an Axum extractor via `FromRequestParts`. Passwords are hashed with Argon2.
- **Serialization** — Derive `Serialize`/`Deserialize` on all types. Use `#[serde(skip_serializing)]` for secrets like password hashes. Use `#[serde(default)]` for optional booleans.
- **Config** — Environment variables loaded via `dotenvy`, parsed manually in a `Config` struct.
- **Tracing** — Use `tracing` and `tracing-subscriber` with env filter.

### Common Crate

- All types derive `Serialize` and `Deserialize`.
- Domain types: `Area`, `Game`, `Player`, `World`, `Stack`, `GameStatus` (enum).
- `Stack` enforces a dice count between 1 and 8 with custom error types.

### Game Client (Bevy)

- Uses Bevy 0.18 ECS architecture (systems, components, resources).
- Dev profile: `opt-level = 1` for the game crate, `opt-level = 3` for dependencies.
- The `common` crate is a path dependency for shared types.

### Tests

- Organise tests inside `#[cfg(test)] mod tests { ... }` at the bottom of each file.
- Use `// ==== Section Name ====` comment headers to group related tests within the test module.
- Aim for comprehensive coverage: valid inputs, invalid inputs, edge cases, and error paths.

---

## Development Workflow

### Test-Driven Development

1. **Write tests first** — Before implementing a feature, write unit tests that describe the expected behaviour.
2. **Unit tests** — Every module should have `#[cfg(test)] mod tests` with thorough coverage.
3. **Integration tests** — Add integration tests for API routes and cross-module interactions.

### After Every Change

Always run the following commands after making changes:

```sh
cargo fmt
cargo clippy
cargo test
```

### Adding Dependencies

Always use `cargo add` to add new crate dependencies:

```sh
cargo add <crate> -p <package>         # add to a specific workspace member
cargo add <crate> -p <package> -F feat # with features
```

### WASM Build & Release

The WASM build artifacts are **not committed to source control** — they are generated cleanly each time. Use the helper script to build the game crate to WebAssembly:

```sh
./scripts/build-wasm.sh
```

This runs `wasm-pack build` against the `game` crate in release mode and outputs the artifacts to `frontend/src/wasm/`.

---

## Database

- **Engine**: PostgreSQL 18 (Docker Compose in `backend/compose.yml`).
- **Migrations**: SQLx migrations in `backend/migrations/`. Run with `sqlx migrate run`.
- **Naming**: Snake-case table and column names. UUID primary keys.
- **Local development**: The PostgreSQL container must be running when building locally or preparing SQLx statements. Start it with `docker compose -f backend/compose.yml up -d`.
- **Offline mode**: After making changes to any database queries, run `cargo sqlx prepare -p backend` to regenerate the query metadata. This allows CI to build with `SQLX_OFFLINE=true` without a live database connection.

---

## Git & GitHub

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <short summary>
```

Common types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `ci`, `build`.
Scope is optional but encouraged (e.g. `backend`, `game`, `common`, `frontend`).

Examples:

```
feat(backend): add WebSocket game lobby endpoint
fix(common): clamp stack dice count on deserialization
docs: update README with build prerequisites
test(backend): add integration tests for user registration
```

### Pull Requests

Use the **GitHub CLI** (`gh`) to create pull requests:

```sh
gh pr create --title "<conventional title>" --body "<description>"
```

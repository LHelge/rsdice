# rsdice

A turn-based online multiplayer dice game.

Players compete on a map of hexagonal tiles grouped into areas. Each game supports up to **6 players**. Each area holds 1–8 dice. On your turn you attack adjacent enemy areas — the attacker and defender each roll the dice on their area. If the attacker's total is strictly greater, they capture the area; otherwise the attacker loses all dice except one. At the end of your turn you receive bonus dice equal to your largest group of connected areas. Excess dice (up to 60) are stored for future turns. The last player standing wins.

## Architecture

```
rsdice/                 # Cargo workspace (resolver = "3")
├── backend/            # Axum REST API + WebSocket server
├── common/             # Shared game domain types
├── game/               # Bevy game client (compiles to WASM)
└── frontend/           # Vite + TypeScript + React (hosts WASM build)
```

| Layer | Technology |
|---|---|
| Backend | Rust 2024, Axum 0.8, SQLx, PostgreSQL, JWT + Argon2 |
| Game client | Rust 2024, Bevy 0.18, WebAssembly |
| Common | Rust 2024 shared crate |
| Frontend | Node, Vite, TypeScript, React |
| Communication | WebSockets |

## Prerequisites

- **Rust** (edition 2024) — install via [rustup](https://rustup.rs/)
- **wasm-pack** — `cargo install wasm-pack`
- **Docker & Docker Compose** — for the PostgreSQL database
- **Node.js** (LTS) — for the frontend
- **SQLx CLI** — `cargo install sqlx-cli --features postgres`

## Getting Started

### 1. Start the database

```sh
docker compose -f backend/compose.yml up -d
```

### 2. Run database migrations

```sh
cd backend
sqlx migrate run
cd ..
```

### 3. Configure environment variables

Create a `.env` file in the `backend/` directory:

```env
PORT=3000
JWT_SECRET=your-secret-key
DATABASE_URL=postgres://rsdice:rsdice@localhost:5432/rsdice
```

### 4. Run the backend

```sh
cargo run -p backend
```

### 5. Build the game client (WASM)

```sh
./scripts/build-wasm.sh
```

### 6. Run the frontend

```sh
cd frontend
npm install
npm run dev
```

## Development

### Running checks

Always run after making changes:

```sh
cargo fmt
cargo clippy
cargo test
```

### SQLx offline mode

After modifying database queries, regenerate query metadata for CI builds:

```sh
cargo sqlx prepare -p backend
```

### Adding dependencies

Use `cargo add` to keep versions current:

```sh
cargo add <crate> -p <package>
```

## License

See [LICENSE](LICENSE) for details.

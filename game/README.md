# rsdice — Game Client

The interactive game client for rsdice, built with Bevy and compiled to WebAssembly to run in the browser.

## Tech Stack

- **Rust 2024** with **Bevy 0.18** (ECS game engine)
- **WebAssembly** via `wasm-bindgen`
- **common** crate for shared domain types
- Communicates with the backend over **WebSockets**

## Prerequisites

- **Rust** (edition 2024) — install via [rustup](https://rustup.rs/)
- **wasm32-unknown-unknown target** — `rustup target add wasm32-unknown-unknown`
- **wasm-bindgen-cli** — `cargo install wasm-bindgen-cli`

## Development

Run natively for quick iteration (Bevy window):

```sh
cargo run -p game
```

### Dev profile

The game crate uses `opt-level = 1` in dev mode for faster compile times, while dependencies (including Bevy) are compiled with `opt-level = 3` for acceptable runtime performance.

## Building for WASM

Compile to WebAssembly and prepare for the frontend:

```sh
cargo build -p game --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir frontend/public/wasm --target web target/wasm32-unknown-unknown/release/game.wasm
```

The generated files in `frontend/public/wasm/` are then served by the Vite frontend.

## Project Structure

```
game/
└── src/
    └── main.rs    # Bevy app entry point — systems, components, resources
```

## Testing

```sh
cargo test -p game
```

# rsdice — Game Client

The interactive game client for rsdice, built with Bevy and compiled to WebAssembly to run in the browser.

## Tech Stack

- **Rust 2024** with **Bevy 0.18** (ECS game engine)
- **WebAssembly** via `wasm-pack`
- **common** crate for shared domain types
- Communicates with the backend over **WebSockets**

## Prerequisites

- **Rust** (edition 2024) — install via [rustup](https://rustup.rs/)
- **wasm-pack** — `cargo install wasm-pack`

## Development

Run natively for quick iteration (Bevy window):

```sh
cargo run -p game
```

### Dev profile

The game crate uses `opt-level = 1` in dev mode for faster compile times, while dependencies (including Bevy) are compiled with `opt-level = 3` for acceptable runtime performance.

## Building for WASM

Use the helper script from the workspace root to build and output to the frontend:

```sh
./scripts/build-wasm.sh
```

This runs `wasm-pack build` in release mode and outputs the artifacts to `frontend/src/wasm/`. These files are **not committed to source control** — they are built cleanly each time.

The generated files are imported by the Vite frontend as ES modules.

## Project Structure

```
game/
├── src/
│   ├── lib.rs     # Shared app builder + WASM entry point
│   └── main.rs    # Native binary entry point (dev builds)
└── Cargo.toml     # [lib] crate-type = ["cdylib", "rlib"]
```

## Testing

```sh
cargo test -p game
```

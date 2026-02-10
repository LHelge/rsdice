# rsdice — Common

Shared game domain types used by both the backend and the Bevy game client.

## Tech Stack

- **Rust 2024**
- **Serde** for serialization / deserialization
- **UUID** for identifiers
- **thiserror** for error types

## Domain Types

| Type | Description |
|---|---|
| `Area` | A group of hexagonal tiles owned by a player, holding a stack of dice |
| `Game` | Top-level game state |
| `Player` | A participant in a game (max 6 per game) |
| `World` | The full map of areas and their adjacency |
| `Stack` | A dice stack (1–8 dice) with enforced bounds |
| `GameStatus` | Enum: `Waiting`, `InProgress`, `Finished` |

All types derive `Serialize` and `Deserialize` for use over WebSocket messages and REST payloads.

The `Color` enum has 6 variants (one per player slot), supports numeric conversion via `#[repr(u8)]` / `TryFrom<u8>`, and provides a `to_hex()` method for hex color codes. The constant `MAX_PLAYERS` is set to 6.

## Usage

This crate is a path dependency for both `backend` and `game`:

```toml
[dependencies]
common = { path = "../common" }
```

## Testing

```sh
cargo test -p common
```

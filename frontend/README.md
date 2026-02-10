# rsdice — Frontend

The web frontend for rsdice, hosting the WASM game client in the browser.

## Tech Stack

- **Node.js** with **Vite** (dev server & build tool)
- **TypeScript** + **React**
- Serves the compiled **WebAssembly** game client from `public/wasm/`

## Prerequisites

- **Node.js** (LTS) — install via [nvm](https://github.com/nvm-sh/nvm) or [nodejs.org](https://nodejs.org/)

## Setup

### 1. Install dependencies

```sh
npm install
```

### 2. Build the WASM game client

The WASM files must be built from the game crate before the frontend can serve them:

```sh
# From the workspace root
cargo build -p game --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir frontend/public/wasm --target web target/wasm32-unknown-unknown/release/game.wasm
```

### 3. Run the dev server

```sh
npm run dev
```

## Project Structure

```
frontend/
├── public/
│   └── wasm/          # WASM build output (generated, not committed)
├── src/               # React application source
└── package.json
```

## Production Build

```sh
npm run build
```

The output in `dist/` includes the React app and the WASM game client, ready to be served by any static file host.

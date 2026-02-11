#!/usr/bin/env bash
# scripts/build-wasm.sh — Build the game crate to WebAssembly using wasm-pack
#
# Usage: build-wasm.sh [--no-opt] [output-dir]
#   --no-opt    Skip wasm-opt optimization (faster CI builds)
#   output-dir  Directory for WASM artifacts (default: frontend/src/wasm)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

EXTRA_ARGS=()
if [[ "${1:-}" == "--no-opt" ]]; then
    EXTRA_ARGS+=("--no-opt")
    shift
fi

OUT_DIR="${1:-$ROOT_DIR/frontend/src/wasm}"

echo "🔨 Building game crate to WebAssembly (release)..."
wasm-pack build "$ROOT_DIR/game" \
    --target web \
    --out-dir "$OUT_DIR" \
    --release \
    "${EXTRA_ARGS[@]}"

# wasm-pack generates a package.json and README.md in the output
# directory; remove them since the output lives inside the frontend source tree.
rm -f "$OUT_DIR/package.json" "$OUT_DIR/README.md"

echo "✅ WASM build complete → $OUT_DIR"

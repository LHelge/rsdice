# rsdice builder image
#
# Base image with all tooling needed to build every crate in the workspace:
#   - Rust toolchain (rustfmt, clippy)
#   - wasm32-unknown-unknown target + wasm-pack
#   - System libraries for Bevy (X11, ALSA/sound)
#   - Node.js 22 (frontend build)

FROM rust:1.93-bookworm

# ---- System packages for Bevy (X11 + sound) and general build tools ----
# https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Build tools
    g++ \
    pkg-config \
    # Bevy core dependencies (Ubuntu)
    libx11-dev \
    libasound2-dev \
    libudev-dev \
    libxkbcommon-x11-0 \
    # Wayland support
    libwayland-dev \
    libxkbcommon-dev \
    && rm -rf /var/lib/apt/lists/*

# ---- Node.js 22 (LTS) ----
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# ---- Rust: WASM target + wasm-pack ----
RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-pack


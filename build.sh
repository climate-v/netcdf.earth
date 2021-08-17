#!/usr/bin/env bash
set -o pipefail
set -o errexit

SCRIPT_DIR=$(dirname $(readlink -f $0))
VISUALIZE_DIR="$SCRIPT_DIR/visualize"

echo "==> Making sure wasm target is installed"
rustup target add wasm32-unknown-unknown

echo "==> Making sure wasm-pack is installed"
cargo install wasm-pack

echo "==> Building wrapper library"
cd "$VISUALIZE_DIR"
wasm-pack build --target no-modules

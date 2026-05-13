#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

cd "$ROOT_DIR/backend"
cargo build --release

cd "$ROOT_DIR/frontend"
npm install
npm run build

echo "release build finished"

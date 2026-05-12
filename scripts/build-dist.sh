#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

cargo build --release --workspace

cp target/release/killian-server dist/bin/killian-server
cp target/release/killian-client dist/bin/killian-client

chmod +x dist/bin/killian-server dist/bin/killian-client

echo "Build concluido em $ROOT_DIR/dist/bin"

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR/web-client"

PORT="${WEB_PORT:-8080}"
HOST="${WEB_HOST:-0.0.0.0}"

exec python3 -m http.server "$PORT" --bind "$HOST"

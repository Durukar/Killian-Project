#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

BIND_ADDR="${KILLIAN_BIND:-${CHAT_BIND:-0.0.0.0:7000}}"

if [ ! -x dist/bin/killian-server ]; then
  echo "Binario nao encontrado em dist/bin/killian-server. Rode scripts/build-dist.sh primeiro."
  exit 1
fi

exec dist/bin/killian-server "$BIND_ADDR"

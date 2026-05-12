#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

if [ ! -x dist/bin/killian-client ]; then
  echo "Binario nao encontrado em dist/bin/killian-client. Rode scripts/build-dist.sh primeiro."
  exit 1
fi

# KILLIAN_SERVER aceita host:porta (ex: 192.168.0.10:7000) ou ws://host:porta
export KILLIAN_SERVER="${KILLIAN_SERVER:-ws://127.0.0.1:7001}"

exec dist/bin/killian-client

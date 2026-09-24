#!/usr/bin/env bash
# build.sh — build tvty (debug), the binary the tvty-wbox app command runs.
#
# Only builds. Does NOT manage the compositor lifecycle — the caller does:
#   kill → build → launch
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
cd "$PROJECT_ROOT"
cargo build 2>&1 | tail -n 40

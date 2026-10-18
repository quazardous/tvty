#!/usr/bin/env bash
# tvty-egui inside the compositor, on the scene TVTY_EGUI_SCENE names
# (default: ../../dev/scene.json, dumped by ../scripts/scene.py).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
bin="${CARGO_TARGET_DIR:-$here/../target}/debug/tvty-egui"
export TVTY_EGUI_SCENE="${TVTY_EGUI_SCENE:-$here/../../dev/scene.json}"
export TVTY_EGUI_STATE="${TVTY_EGUI_STATE:-$here/state}"
mkdir -p "$here/log"
exec "$bin" 2>"$here/log/tvty-egui.log"

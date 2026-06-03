#!/usr/bin/env bash
set -euo pipefail

REPO="miracle-wm-org/miri-plugin"
WASM="miri_plugin.wasm"
PLUGIN_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/miracle-wm/plugins"

echo "Downloading latest $WASM..."
URL="https://github.com/$REPO/releases/download/nightly/$WASM"
mkdir -p "$PLUGIN_DIR"
curl -fsSL "$URL" -o "$PLUGIN_DIR/$WASM"
echo "Installed to $PLUGIN_DIR/$WASM"

#!/usr/bin/env bash
set -euo pipefail

REPO="miracle-wm-org/miri-plugin"
WASM="miri_plugin.wasm"
PLUGIN_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/miracle-wm/plugins"

CHANNEL="${CHANNEL:-stable}"
[[ "${1:-}" == "--nightly" ]] && CHANNEL="nightly"

if [[ "$CHANNEL" == "nightly" ]]; then
  echo "Downloading latest nightly $WASM..."
  URL="https://github.com/$REPO/releases/download/nightly/$WASM"
else
  echo "Downloading latest stable $WASM..."
  URL="https://github.com/$REPO/releases/latest/download/$WASM"
fi

mkdir -p "$PLUGIN_DIR"
curl -fsSL "$URL" -o "$PLUGIN_DIR/$WASM"
echo "Installed to $PLUGIN_DIR/$WASM"

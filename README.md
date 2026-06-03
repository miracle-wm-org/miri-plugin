# miri-plugin
A scrolling window manager plugin for miracle-wm

[Video Demo](https://github.com/user-attachments/assets/3233f932-a0e2-4adc-95a5-4eb0836afd9c)

## Installation

Download and install the latest nightly build:

```sh
curl -fsSL https://raw.githubusercontent.com/miracle-wm-org/miri-plugin/main/install.sh | bash
```

This places `miri_plugin.wasm` in `$XDG_CONFIG_HOME/miracle-wm/plugins` (defaults to `~/.config/miracle-wm/plugins`), which will be automatically loaded by miracle-wm.

## Prerequisites
```sh
sudo apt-get install -y libmircore-dev clang libclang-dev
rustup target add wasm32-wasip1
```

## Build
```sh
cargo build --target wasm32-wasip1 --release

# This will build to:
#    target/wasm32-wasip1/release/miri_plugin.wasm
```

## Usage
```yaml
# ~/.config/miracle-wm/config.yaml

plugins:
 - path: /path/to/miri/target/wasm32-wasip1/release/miri_plugin.wasm
   inner_gap: 50 # Optional
   outer_gap: 5  # Optional
   workspace: 1  # Optional. This makes it so that Miri is only applied to a single specified workspace.
```

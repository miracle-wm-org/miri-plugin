# miri-plugin
A scrolling window manager plugin for miracle-wm

## Installation
```sh
rustup target add wasm32-wasip1
```

## Building
```sh
cargo build --target wasm32-wasip1 --release

# This will build to:
#    target/wasm32-wasip1/release/miri-plugin.wasm
```

## Usage
```yaml
# ~/.config/miracle-wm/config.yaml

plugins:
 - path: /path/to/miri/target/wasm32-wasip1/release/miri-plugin.wasm
   inner_gap: 50 # Optional
   outer_gap: 5  # Optional
```

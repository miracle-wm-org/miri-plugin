# miri-plugin
A scrolling window manager plugin for miracle-wm

## Download

The latest nightly WASM build is available on the [releases page](https://github.com/miracle-wm-org/miri-plugin/releases/tag/nightly):

```
https://github.com/miracle-wm-org/miri-plugin/releases/download/nightly/miri_plugin.wasm
```

## Building from Source

### Installation
```sh
rustup target add wasm32-wasip1
```

### Building
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
```

# miri-plugin
A scrolling window manager plugin for miracle-wm

[Video Demo](https://github.com/user-attachments/assets/3233f932-a0e2-4adc-95a5-4eb0836afd9c)

## Installation

Download and install the latest nightly build:

```sh
curl -fsSL https://raw.githubusercontent.com/miracle-wm-org/miri-plugin/main/install.sh | bash
```

Alternatively, manually add the plugin to your miracle-wm configuration file (`~/.config/miracle-wm/config.yaml`):

```yaml
# ~/.config/miracle-wm/config.yaml

plugins:
 - path: /path/to/miri/target/wasm32-wasip1/release/miri_plugin.wasm
   inner_gap: 50 # Optional
   outer_gap: 5  # Optional
   workspace: 1  # Optional. This makes it so that Miri is only applied to a single specified workspace.
```

### Building

### Prerequisites
```sh
sudo apt-get install -y libmircore-dev clang libclang-dev
rustup target add wasm32-wasip1
```

### Compilation

```sh
cargo build --target wasm32-wasip1 --release
```

The compiled WASM file can be found at `target/wasm32-wasip1/release/miri_plugin.wasm`.

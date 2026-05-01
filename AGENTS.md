# AGENTS.md

## Project

Rust + Slint GUI for Linux USB serial port mapping (`/dev/ttyUSB*`, `/dev/ttyACM*`). Generates udev rules for stable symlinks. Upstream C++ project: [Mq-b/usb_map](https://github.com/Mq-b/usb_map) — only `usb_map` is ported, not `find_4g_module`.

## Build

```bash
# System deps required before cargo will link
sudo apt-get install build-essential pkg-config libudev-dev libx11-dev libxkbcommon-dev libwayland-dev libfontconfig1-dev

cargo build --locked --release
cargo test --locked
```

CI does **not** run `cargo check` or `cargo clippy` — don't treat those as gates.

## Slint specifics

- `build.rs` compiles `ui/app_window.slint` with `fluent` style, generating Rust types (`MainWindow`, `DeviceRowData`) via `slint::include_modules!()` in `main.rs`
- Backend forced to `winit` through `.cargo/config.toml` env `SLINT_BACKEND=winit`
- Slint crate uses `default-features = false` — only `backend-winit`, `renderer-software`, `compat-1-2` are enabled
- UI strings are all Chinese (Simplified)

## Non-obvious details

- **Rule format** uses `SUBSYSTEM=="tty"` (not `ttyUSB*`) to cover both ttyUSB and ttyACM — this is a deliberate departure from the upstream C++
- **Default rule file** path `/etc/udev/rules.d/relia.rules` is hardcoded in `models.rs:3`
- **Validation** (`models.rs:84-101`): virtual name must be non-empty and contain no slashes; physical ID must not be `"N/A"`
- **Tests** in `rule_file.rs` write real temp files (nanosecond-unique names in system temp dir)

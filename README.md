# 🐱 Lynx Installer

A modern, cross-platform installer framework with fully customizable UI.
Build beautiful installers with animated splash screens, custom themes,
and smooth progress animations — for Windows, macOS, and Linux.

---

## Workspace Structure

```
lynx-installer/
├── Cargo.toml          — workspace root (shared deps + profiles)
├── engine/             — core Rust library (config, bundling, extraction, IPC)
├── shell/              — installer UI shell (Tauri + WebView, what end users see)
└── builder/            — builder desktop app (Tauri + React, what YOU use)
```

## Crates

| Crate | Description |
|---|---|
| `lynx-engine` | Core library. Parses `.lynx` configs, bundles payloads into `.lynxpak` archives, extracts files, runs install steps, reports progress. |
| `lynx-shell` | The thin native window that wraps the WebView UI during installation. Receives progress events from the engine via IPC and drives animations. |
| `lynx-builder` | The desktop app you use to create `.lynx` installer projects. Visual editor, theme picker, build button. |

## Quick Start

```bash
# Build everything
cargo build

# Run all tests
cargo test

# Build in release mode
cargo build --release

# Test a specific crate
cargo test -p lynx-engine
```

## The `.lynx` Project File

Installers are defined by a `.lynx` TOML file:

```toml
[app]
name      = "My App"
version   = "1.0.0"
publisher = "Acme Corp"
id        = "com.acme.myapp"

[theme]
name         = "lynx-default"
path         = "themes/lynx-default"
accent_color = "#FF6B35"

[[files]]
source      = "dist/"
destination = "{install_dir}"
recursive   = true

[[steps]]
kind  = "extract"
label = "Installing files..."

[[steps]]
kind      = "shortcut"
label     = "Creating shortcuts..."
target    = "{install_dir}/myapp.exe"
locations = ["desktop", "start_menu"]
```

## Themes

Themes are folders of HTML/CSS/JS that control what the installer looks like.
Drop a theme folder into your project and reference it in your `.lynx` file.
The engine injects progress events into the WebView via a simple JS bridge.

Built-in themes:
- `lynx-default` — dark, geometric, orange/magenta gradient (matches Lynx branding)

## License

MIT
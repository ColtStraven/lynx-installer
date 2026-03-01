<div align="center">
  <h1>🐺 Lynx Installer</h1>
  <p><strong>A modern, lightweight installer builder for Windows applications.</strong></p>
  <p>
    <img src="https://img.shields.io/badge/platform-Windows-blue?style=flat-square" />
    <img src="https://img.shields.io/badge/license-Proprietary-red?style=flat-square" />
    <img src="https://img.shields.io/badge/made_by-Stray_Helix-orange?style=flat-square" />
  </p>
</div>

---

Lynx Installer is a desktop tool for creating professional Windows installers without the complexity of NSIS, Inno Setup, or WiX. Define your app, add your files, configure install steps, and build a single `.exe` that users can just run.

## Features

- 🎨 **Custom installer UI** — frameless, themed installer window your users actually see
- 📦 **Single-file output** — everything bundled into one redistributable `.exe`
- 🛡 **Uninstaller built-in** — registers in Windows Add/Remove Programs automatically  
- ⚙ **Install steps** — extract files, create shortcuts, write registry keys, run commands
- 🖼 **App icon support** — your icon embedded in the installer exe
- 🔧 **Builder UI** — visual project editor, no config files to hand-write
- ✅ **Inline validation** — catch errors before you build, not after

## Download

Grab the latest release from the [Releases](../../releases) page. You'll get:

- `lynx-builder.exe` — the visual project editor
- `lynx-build.exe` — the CLI build tool (used by the builder internally)

No installation required — just run `lynx-builder.exe`.

## Quick Start

### 1. Create a project

Open `lynx-builder.exe` and fill in the **App Info** section:

| Field | Example |
|-------|---------|
| App Name | My Application |
| Version | 1.0.0 |
| App ID | com.example.myapp |
| Publisher | Acme Corp |

### 2. Add your files

Go to the **Files** section and add the files or folders you want installed. Each entry has a source path (on your machine) and a destination path (where it lands on the user's machine). Use tokens like `{install_dir}` for the destination.

### 3. Configure install steps

Go to the **Steps** section and add steps in order:

| Step | What it does |
|------|-------------|
| `Extract` | Unpacks your files to the install directory |
| `Shortcut` | Creates a desktop or Start Menu shortcut |
| `Registry` | Writes a registry key |
| `Command` | Runs an executable or script |
| `Register Uninstaller` | Adds your app to Add/Remove Programs |

A typical setup is: **Extract → Shortcut → Register Uninstaller**.

### 4. Set your icon (optional)

In **App Info**, click **Browse** next to Icon and select a `.ico` file. This icon will be embedded in the installer exe and shown in Add/Remove Programs.

### 5. Build

Go to the **Build** section:

1. Set the **Shell directory** to the `shell/` folder in this repo
2. Click **▶ Build Installer**
3. Watch the progress bar — the Rust compile step takes 2–4 minutes on first build, ~30s with cache

Your installer `.exe` will be saved next to your `.lynx` project file.

## Project File Format

Projects are saved as `.lynx` files (TOML format). You can open them in any text editor.

```toml
[app]
name = "My Application"
version = "1.0.0"
id = "com.example.myapp"
publisher = "Acme Corp"
default_install_dir = "{program_files}/{app_name}"

[[steps]]
kind = "extract"
label = "Installing files..."

[[steps]]
kind = "shortcut"
label = "Creating shortcuts..."
target = "{install_dir}/MyApp.exe"
locations = ["desktop", "start_menu"]

[[steps]]
kind = "register_uninstaller"
label = "Registering uninstaller..."
```

## Install Directory Tokens

Use these tokens in paths — they resolve to the correct location on the user's machine:

| Token | Resolves to |
|-------|-------------|
| `{install_dir}` | The directory the user chose during install |
| `{program_files}` | `C:\Program Files` |
| `{local_app_data}` | `C:\Users\<user>\AppData\Local` |
| `{app_data}` | `C:\Users\<user>\AppData\Roaming` |
| `{temp}` | `C:\Users\<user>\AppData\Local\Temp` |
| `{app_name}` | Your app's name (spaces replaced with hyphens) |
| `{app_version}` | Your app's version string |

## Building from Source

> ⚠ Source use is restricted — see [LICENSE](LICENSE).

Requirements:
- Rust 1.80+
- Node.js 18+
- Windows 10/11

```bash
# Build the engine (includes lynx-build and lynx-uninstaller)
cargo build -p lynx-engine --release

# Build the builder UI
cd builder
cargo tauri build
```

## License

Copyright © 2026 Stray Helix. All rights reserved.

Free to download and use. Source code is provided for transparency only.
See [LICENSE](LICENSE) for full terms.
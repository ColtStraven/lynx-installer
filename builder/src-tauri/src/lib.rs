// lib.rs
// Lynx Builder — Tauri backend
//
// Exposes commands to the React UI for:
//   - Creating, loading, and saving .lynx projects
//   - Browsing the filesystem for files/directories
//   - Running the bundler to produce .lynxpak output
//   - Launching the shell for preview

use tauri::{AppHandle, Emitter, Manager};
use lynx_engine::LynxProject;

// ─────────────────────────────────────────────
//  Window commands
// ─────────────────────────────────────────────

#[tauri::command]
fn shell_ready(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn shell_close(app: AppHandle) { app.exit(0); }

#[tauri::command]
fn shell_minimize(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") { let _ = w.minimize(); }
}

#[tauri::command]
fn shell_maximize(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_maximized().unwrap_or(false) {
            let _ = w.unmaximize();
        } else {
            let _ = w.maximize();
        }
    }
}
use std::path::PathBuf;

// ─────────────────────────────────────────────
//  Project commands
// ─────────────────────────────────────────────

/// Create a new empty project with sensible defaults
#[tauri::command]
fn new_project() -> serde_json::Value {
    serde_json::json!({
        "app": {
            "name": "My App",
            "version": "1.0.0",
            "publisher": "",
            "id": "com.example.myapp",
            "description": "",
            "icon": null,
            "default_install_dir": "{program_files}/{app_name}",
            "url": null,
        },
        "theme": {
            "name": "lynx-default",
            "path": "themes/lynx-default",
            "accent_color": "#FF6B35",
            "background_color": "#1A1A2E",
            "custom_vars": {}
        },
        "files": [],
        "steps": [
            { "kind": "extract", "label": "Installing application files..." },
            { "kind": "shortcut", "label": "Creating shortcuts...", "target": "{install_dir}/{app_name}.exe", "locations": ["desktop", "start_menu"], "name": null },
            { "kind": "register_uninstaller", "label": "Finalizing installation..." }
        ],
        "prerequisites": [],
        "uninstall": {
            "enabled": true,
            "exe_name": "uninstall.exe",
            "remove_install_dir": true,
            "extra_remove_paths": []
        }
    })
}

/// Load a .lynx project from disk
#[tauri::command]
fn load_project(path: String) -> Result<serde_json::Value, String> {
    let project_path = PathBuf::from(&path);

    let project = LynxProject::from_file(&project_path)
        .map_err(|e| format!("Failed to load project: {e}"))?;

    serde_json::to_value(&project)
        .map_err(|e| format!("Failed to serialize project: {e}"))
}

/// Save a .lynx project to disk
#[tauri::command]
fn save_project(path: String, project_json: serde_json::Value) -> Result<(), String> {
    let project: LynxProject = serde_json::from_value(project_json)
        .map_err(|e| format!("Invalid project data: {e}"))?;

    let toml = project.to_toml_str()
        .map_err(|e| format!("Failed to serialize to TOML: {e}"))?;

    std::fs::write(&path, toml)
        .map_err(|e| format!("Failed to write file: {e}"))?;

    Ok(())
}

/// Validate a project and return any errors/warnings
#[tauri::command]
fn validate_project(project_json: serde_json::Value) -> serde_json::Value {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Check required app fields
    if let Some(app) = project_json.get("app") {
        if app.get("name").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
            errors.push("App name is required".into());
        }
        if app.get("version").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
            errors.push("App version is required".into());
        }
        if app.get("publisher").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
            warnings.push("Publisher name is recommended".into());
        }
        if app.get("id").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
            errors.push("App ID is required (e.g. com.example.myapp)".into());
        }
    } else {
        errors.push("Missing app configuration".into());
    }

    // Check files
    if let Some(files) = project_json.get("files").and_then(|v| v.as_array()) {
        if files.is_empty() {
            warnings.push("No files added to bundle — the installer won't install anything".into());
        }
    }

    // Check steps
    if let Some(steps) = project_json.get("steps").and_then(|v| v.as_array()) {
        if steps.is_empty() {
            errors.push("No install steps defined".into());
        }
        let has_extract = steps.iter().any(|s| {
            s.get("kind").and_then(|v| v.as_str()) == Some("extract")
        });
        if !has_extract {
            warnings.push("No extract step — files won't be extracted during install".into());
        }
    }

    serde_json::json!({
        "valid": errors.is_empty(),
        "errors": errors,
        "warnings": warnings
    })
}

// ─────────────────────────────────────────────
//  Build commands
// ─────────────────────────────────────────────

/// Run the full lynx-build pipeline — produces a single installer .exe
#[tauri::command]
async fn build_installer(
    app: AppHandle,
    project_path: String,
    shell_path: String,
    output_path: String,
) -> Result<serde_json::Value, String> {
    let app_clone = app.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        // Find lynx-build binary next to this executable
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Cannot find current exe: {e}"))?;
        let exe_dir = current_exe.parent()
            .ok_or("Cannot find exe directory")?;

        // Try common locations for lynx-build
        let candidates = [
            exe_dir.join("lynx-build.exe"),
            exe_dir.join("lynx-build"),
            // Dev: workspace target/debug
            PathBuf::from("target/debug/lynx-build.exe"),
            PathBuf::from("target/debug/lynx-build"),
            PathBuf::from("target/release/lynx-build.exe"),
        ];

        let lynx_build = candidates.iter()
            .find(|p| p.exists())
            .ok_or_else(|| "lynx-build not found. Run: cargo build -p lynx-engine --bin lynx-build".to_string())?
            .clone();

        let _ = app_clone.emit("build-progress", serde_json::json!({
            "type": "info",
            "message": format!("Using lynx-build: {}", lynx_build.display())
        }));

        // Create output directory if needed
        if let Some(parent) = PathBuf::from(&output_path).parent() {
            std::fs::create_dir_all(parent).ok();
        }

        // Spawn lynx-build and stream output line by line
        let mut child = std::process::Command::new(&lynx_build)
            .args([
                "--project", &project_path,
                "--shell",   &shell_path,
                "--output",  &output_path,
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn lynx-build: {e}"))?;

        // Capture stderr in a background thread
        let stderr_handle = child.stderr.take().map(|stderr| {
            std::thread::spawn(move || {
                use std::io::Read;
                let mut buf = String::new();
                std::io::BufReader::new(stderr).read_to_string(&mut buf).ok();
                buf
            })
        });

        // Stream stdout lines as progress events
        if let Some(stdout) = child.stdout.take() {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                let _ = app_clone.emit("build-progress", serde_json::json!({
                    "type": "info",
                    "message": line
                }));
            }
        }

        let status = child.wait()
            .map_err(|e| format!("Failed to wait for lynx-build: {e}"))?;

        let stderr_output = stderr_handle
            .and_then(|h| h.join().ok())
            .unwrap_or_default();

        if !status.success() {
            return Err(format!("lynx-build failed:\n{}", stderr_output.trim()));
        }

        let size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(serde_json::json!({
            "success": true,
            "output_path": output_path,
            "size_bytes": size,
        }))
    })
    .await
    .map_err(|e| format!("Task failed: {e}"))?;

    result
}

/// Get metadata about the current workspace
#[tauri::command]
fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "engine_version": "0.1.0",
        "supported_platforms": ["windows", "macos", "linux"]
    })
}

// ─────────────────────────────────────────────
//  Dialog commands
// ─────────────────────────────────────────────

/// Open a native file open dialog for .lynx files
#[tauri::command]
async fn pick_lynx_file(app: AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx = std::sync::Mutex::new(Some(tx));
    app.dialog()
        .file()
        .add_filter("Lynx Project", &["lynx"])
        .pick_file(move |f| {
            if let Some(tx) = tx.lock().unwrap().take() {
                let _ = tx.send(f.map(|p| p.to_string()));
            }
        });
    rx.await.ok().flatten()
}

/// Open a native save dialog for .lynx files
#[tauri::command]
async fn save_lynx_file(app: AppHandle, default_name: String) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx = std::sync::Mutex::new(Some(tx));
    app.dialog()
        .file()
        .add_filter("Lynx Project", &["lynx"])
        .set_file_name(&default_name)
        .save_file(move |f| {
            if let Some(tx) = tx.lock().unwrap().take() {
                let _ = tx.send(f.and_then(|p| p.into_path().ok()).map(|p| p.to_string_lossy().to_string()));
            }
        });
    rx.await.ok().flatten()
}

/// Open a native save dialog for the output .exe
#[tauri::command]
async fn save_exe_file(app: AppHandle, default_name: String) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx = std::sync::Mutex::new(Some(tx));
    app.dialog()
        .file()
        .add_filter("Executable", &["exe"])
        .set_file_name(&default_name)
        .save_file(move |f| {
            if let Some(tx) = tx.lock().unwrap().take() {
                let _ = tx.send(f.and_then(|p| p.into_path().ok()).map(|p| p.to_string_lossy().to_string()));
            }
        });
    rx.await.ok().flatten()
}

/// Open a native folder picker
#[tauri::command]
async fn pick_directory(app: AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx = std::sync::Mutex::new(Some(tx));
    app.dialog()
        .file()
        .pick_folder(move |f| {
            if let Some(tx) = tx.lock().unwrap().take() {
                let _ = tx.send(f.and_then(|p| p.into_path().ok()).map(|p| p.to_string_lossy().to_string()));
            }
        });
    rx.await.ok().flatten()
}

// ─────────────────────────────────────────────
//  App entry point
// ─────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Register plugins
            app.handle().plugin(tauri_plugin_dialog::init())?;
            app.handle().plugin(tauri_plugin_fs::init())?;
            app.handle().plugin(tauri_plugin_shell::init())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            shell_ready,
            shell_close,
            shell_minimize,
            shell_maximize,
            new_project,
            load_project,
            save_project,
            validate_project,
            build_installer,
            get_app_info,
            pick_lynx_file,
            save_lynx_file,
            save_exe_file,
            pick_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lynx Builder");
}
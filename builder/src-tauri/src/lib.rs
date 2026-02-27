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

/// Run the bundler to produce a .lynxpak from the project
#[tauri::command]
async fn build_pak(
    app: AppHandle,
    project_path: String,
    output_path: String,
) -> Result<serde_json::Value, String> {
    let app_clone = app.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let project_path = PathBuf::from(&project_path);
        let output_path  = PathBuf::from(&output_path);

        let project_root = project_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();

        let project = LynxProject::from_file(&project_path)
            .map_err(|e| format!("Failed to load project: {e}"))?;

        let sender = lynx_engine::ProgressSender::new(move |event| {
            let _ = app_clone.emit("build-progress", event.to_json());
        });

        let bundler = lynx_engine::bundle::Bundler::new(&project, project_root, &sender);

        bundler.bundle(&output_path)
            .map(|manifest| serde_json::json!({
                "success": true,
                "file_count": manifest.file_count,
                "total_bytes": manifest.total_bytes,
                "output_path": output_path.to_string_lossy(),
            }))
            .map_err(|e| format!("Bundle failed: {e}"))
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
            build_pak,
            get_app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lynx Builder");
}
// lib.rs
// Lynx Installer Shell — Tauri application entry point

use tauri::{AppHandle, Emitter, Manager};

// Embed the .lynxpak bundle at compile time
// In production the builder replaces this path with the real payload
static EMBEDDED_PAK: &[u8] = include_bytes!("../../../test-payload/test.lynxpak");

// Uninstaller binary — compiled by lynx-build and placed in resources/
// include_bytes! is conditional on the with_uninstaller feature
#[cfg(feature = "with_uninstaller")]
static EMBEDDED_UNINSTALLER: &[u8] = include_bytes!("../resources/uninstall.exe");
#[cfg(not(feature = "with_uninstaller"))]
static EMBEDDED_UNINSTALLER: &[u8] = &[];

// ─────────────────────────────────────────────
//  Embedded project config
// ─────────────────────────────────────────────

fn embedded_project_toml() -> String {
    vec![
        "[app]",
        "name = \"My Awesome App\"",
        "version = \"2.1.0\"",
        "publisher = \"Acme Corp\"",
        "id = \"com.acme.myawesomeapp\"",
        "description = \"The most awesome app ever made.\"",
        "default_install_dir = \"{local_app_data}/MyAwesomeApp\"",
        "",
        "[theme]",
        "name = \"lynx-default\"",
        "path = \"themes/lynx-default\"",
        "accent_color = \"#FF6B35\"",
        "",
        "[[steps]]",
        "kind = \"extract\"",
        "label = \"Installing application files...\"",
    ].join("\n")
}

// ─────────────────────────────────────────────
//  Tauri commands
// ─────────────────────────────────────────────

#[tauri::command]
fn shell_ready(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn shell_close(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn shell_minimize(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
}

#[tauri::command]
fn get_install_config() -> serde_json::Value {
    match lynx_engine::LynxProject::from_toml_str(&embedded_project_toml()) {
        Ok(project) => serde_json::json!({
            "app": {
                "name": project.app.name,
                "version": project.app.version,
                "publisher": project.app.publisher,
                "description": project.app.description,
                "icon": project.app.icon,
            },
            "theme": {
                "accent_color": project.theme.accent_color,
                "background_color": project.theme.background_color,
            },
            "default_install_dir": project.app.default_install_dir,
            "total_steps": project.steps.len(),
        }),
        Err(e) => {
            log::error!("Failed to parse embedded project: {e}");
            serde_json::json!({ "error": e.to_string() })
        }
    }
}

#[tauri::command]
async fn start_install(app: AppHandle, install_dir: String) -> Result<(), String> {
    let app_clone = app.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // Debug: confirm pak is loaded and install dir is correct
        log::info!("PAK size: {} bytes", EMBEDDED_PAK.len());
        log::info!("Install dir: {}", install_dir);

        let toml = embedded_project_toml();
        let project = match lynx_engine::LynxProject::from_toml_str(&toml) {
            Ok(p) => p,
            Err(e) => {
                let _ = app_clone.emit("progress", serde_json::json!({
                    "type": "failed",
                    "step_index": null,
                    "step_label": null,
                    "error": format!("Failed to parse project config: {e}")
                }));
                return;
            }
        };

        let app_for_sender = app_clone.clone();
        let sender = lynx_engine::ProgressSender::new(move |event| {
            let json = event.to_json();
            let value: serde_json::Value = serde_json::from_str(&json)
                .unwrap_or(serde_json::json!({ "type": "warning", "message": "bad event" }));
            let _ = app_for_sender.emit("progress", value);
        });

        let config = lynx_engine::RunnerConfig::new(&install_dir)
            .with_pak_bytes(EMBEDDED_PAK.to_vec())
            .with_uninstaller_bytes(EMBEDDED_UNINSTALLER.to_vec());
        let runner = lynx_engine::InstallerRunner::new(&project, config, &sender);

        if let Err(e) = runner.run() {
            let _ = app_clone.emit("progress", serde_json::json!({
                "type": "failed",
                "step_index": null,
                "step_label": null,
                "error": e.to_string()
            }));
        }
    });

    Ok(())
}

// ─────────────────────────────────────────────
//  App entry point
// ─────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "windows")]
                let _ = window.set_shadow(true);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            shell_ready,
            shell_close,
            shell_minimize,
            get_install_config,
            start_install,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lynx shell");
}
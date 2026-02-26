/// runner.rs
///
/// The InstallerRunner takes a LynxProject and executes each
/// InstallStep in order, reporting progress via ProgressSender.
///
/// This is what the shell calls when the user clicks "Install Now".

use std::path::PathBuf;
use std::time::Instant;

use crate::config::{InstallStep, LynxProject, ShortcutLocation};
use crate::error::{LynxError, LynxResult};
use crate::extract::{Extractor, TokenResolver};
use crate::progress::{ProgressEvent, ProgressSender};

// ─────────────────────────────────────────────
//  Runner config (runtime, not project config)
// ─────────────────────────────────────────────

/// Runtime configuration passed to the runner at install time.
/// Separate from LynxProject because these are user choices made
/// during installation (e.g. the install directory they picked).
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// The directory the user chose to install into
    pub install_dir: String,

    /// Path to the .lynxpak bundle file to extract
    /// In production this will be a temp file extracted from the exe
    pub pak_path: Option<PathBuf>,

    /// Raw pak bytes if embedded directly in the binary
    pub pak_bytes: Option<Vec<u8>>,
}

impl RunnerConfig {
    pub fn new(install_dir: impl Into<String>) -> Self {
        Self {
            install_dir: install_dir.into(),
            pak_path: None,
            pak_bytes: None,
        }
    }

    pub fn with_pak_path(mut self, path: PathBuf) -> Self {
        self.pak_path = Some(path);
        self
    }

    pub fn with_pak_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.pak_bytes = Some(bytes);
        self
    }
}

// ─────────────────────────────────────────────
//  InstallerRunner
// ─────────────────────────────────────────────

pub struct InstallerRunner<'a> {
    project: &'a LynxProject,
    config: RunnerConfig,
    sender: &'a ProgressSender,
}

impl<'a> InstallerRunner<'a> {
    pub fn new(
        project: &'a LynxProject,
        config: RunnerConfig,
        sender: &'a ProgressSender,
    ) -> Self {
        Self { project, config, sender }
    }

    /// Execute all install steps in order.
    /// Emits ProgressEvents throughout.
    pub fn run(&self) -> LynxResult<()> {
        let start = Instant::now();
        let total_steps = self.project.steps.len();

        self.sender.send(ProgressEvent::Started {
            app_name: self.project.app.name.clone(),
            app_version: self.project.app.version.clone(),
            total_steps,
        });

        // Build the token resolver with the user's chosen install dir
        let resolver = TokenResolver::with_defaults(
            &self.project.app.name,
            &self.config.install_dir,
        );

        // Execute each step
        for (index, step) in self.project.steps.iter().enumerate() {
            self.execute_step(index, step, &resolver)?;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        self.sender.send(ProgressEvent::Complete {
            install_dir: self.config.install_dir.clone(),
            duration_ms,
        });

        Ok(())
    }

    // ── Step dispatch ────────────────────────────────────────────────

    fn execute_step(
        &self,
        index: usize,
        step: &InstallStep,
        resolver: &TokenResolver,
    ) -> LynxResult<()> {
        match step {
            InstallStep::Extract { label } => {
                self.run_extract(index, label, resolver)
            }
            InstallStep::Shortcut { label, target, locations, name } => {
                self.run_shortcut(index, label, target, locations, name.as_deref(), resolver)
            }
            InstallStep::Registry { label, hive, key, value_name, value_data, value_type } => {
                self.run_registry(index, label, hive, key, value_name, value_data, value_type, resolver)
            }
            InstallStep::Command { label, command, args, wait, fail_on_error } => {
                self.run_command(index, label, command, args, *wait, *fail_on_error, resolver)
            }
            InstallStep::EnvVar { label, name, value, scope } => {
                self.run_env_var(index, label, name, value, scope, resolver)
            }
            InstallStep::RegisterUninstaller { label } => {
                self.run_register_uninstaller(index, label, resolver)
            }
        }
    }

    // ── Extract ──────────────────────────────────────────────────────

    fn run_extract(
        &self,
        index: usize,
        label: &str,
        resolver: &TokenResolver,
    ) -> LynxResult<()> {
        self.sender.send(ProgressEvent::StepBegin {
            step_index: index,
            step_label: label.to_string(),
        });

        let extractor = Extractor::new(self.sender, resolver.clone(), index);

        if let Some(pak_path) = &self.config.pak_path {
            extractor.extract_from_file(pak_path)?;
        } else if let Some(pak_bytes) = &self.config.pak_bytes {
            extractor.extract_from_bytes(pak_bytes)?;
        } else {
            // No payload — skip with a warning (useful in dev/test)
            self.sender.send(ProgressEvent::StepSkipped {
                step_index: index,
                step_label: label.to_string(),
                reason: "No payload bundle found — skipping extract step".to_string(),
            });
            return Ok(());
        }

        self.sender.send(ProgressEvent::StepComplete {
            step_index: index,
            step_label: label.to_string(),
        });

        Ok(())
    }

    // ── Shortcut ─────────────────────────────────────────────────────

    fn run_shortcut(
        &self,
        index: usize,
        label: &str,
        target: &str,
        locations: &[ShortcutLocation],
        name: Option<&str>,
        resolver: &TokenResolver,
    ) -> LynxResult<()> {
        self.sender.send(ProgressEvent::StepBegin {
            step_index: index,
            step_label: label.to_string(),
        });

        let resolved_target = resolver.resolve(target);
        let shortcut_name = name.unwrap_or(&self.project.app.name);

        for location in locations {
            self.create_shortcut(&resolved_target, shortcut_name, location, resolver)?;
        }

        self.sender.send(ProgressEvent::StepComplete {
            step_index: index,
            step_label: label.to_string(),
        });

        Ok(())
    }

    fn create_shortcut(
        &self,
        target: &str,
        name: &str,
        location: &ShortcutLocation,
        _resolver: &TokenResolver,
    ) -> LynxResult<()> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;

            let dest_dir = match location {
                ShortcutLocation::Desktop => {
                    dirs::desktop_dir()
                        .ok_or_else(|| LynxError::StepFailed {
                            step: "shortcut".into(),
                            reason: "Cannot find desktop directory".into(),
                        })?
                }
                ShortcutLocation::StartMenu => {
                    dirs::data_dir()
                        .ok_or_else(|| LynxError::StepFailed {
                            step: "shortcut".into(),
                            reason: "Cannot find start menu directory".into(),
                        })?
                        .join("Microsoft\\Windows\\Start Menu\\Programs")
                }
                ShortcutLocation::Taskbar => {
                    // Taskbar pinning requires COM automation — skip for now
                    self.sender.send(ProgressEvent::Warning {
                        message: "Taskbar pinning not yet supported, skipping".to_string(),
                    });
                    return Ok(());
                }
            };

            let lnk_path = dest_dir.join(format!("{}.lnk", name));

            // Use PowerShell to create the .lnk shortcut
            let ps_script = format!(
                "$ws = New-Object -ComObject WScript.Shell; \
                 $s = $ws.CreateShortcut('{lnk}'); \
                 $s.TargetPath = '{target}'; \
                 $s.Save()",
                lnk = lnk_path.to_string_lossy().replace('\'', "''"),
                target = target.replace('\'', "''"),
            );

            let status = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
                .status()
                .map_err(|e| LynxError::StepFailed {
                    step: "shortcut".into(),
                    reason: format!("PowerShell failed: {e}"),
                })?;

            if !status.success() {
                return Err(LynxError::StepFailed {
                    step: "shortcut".into(),
                    reason: format!("PowerShell exited with status: {status}"),
                });
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: create an alias in the Applications folder
            // Full implementation comes in a later step
            self.sender.send(ProgressEvent::Warning {
                message: "macOS shortcuts not yet implemented".to_string(),
            });
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: create a .desktop file
            let dest_dir = match location {
                ShortcutLocation::Desktop => {
                    dirs::desktop_dir().unwrap_or_else(|| {
                        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("Desktop")
                    })
                }
                _ => PathBuf::from("/usr/share/applications"),
            };

            std::fs::create_dir_all(&dest_dir)?;

            let desktop_content = format!(
                "[Desktop Entry]\nType=Application\nName={name}\nExec={target}\nTerminal=false\n"
            );

            std::fs::write(
                dest_dir.join(format!("{}.desktop", name.to_lowercase().replace(' ', "-"))),
                desktop_content,
            )?;
        }

        Ok(())
    }

    // ── Registry (Windows only) ──────────────────────────────────────

    fn run_registry(
        &self,
        index: usize,
        label: &str,
        hive: &str,
        key: &str,
        value_name: &str,
        value_data: &str,
        value_type: &str,
        resolver: &TokenResolver,
    ) -> LynxResult<()> {
        self.sender.send(ProgressEvent::StepBegin {
            step_index: index,
            step_label: label.to_string(),
        });

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;

            let resolved_data = resolver.resolve(value_data);
            let full_key = format!("{}\\{}", hive, key);

            let status = Command::new("reg")
                .args([
                    "add", &full_key,
                    "/v", value_name,
                    "/t", value_type,
                    "/d", &resolved_data,
                    "/f",
                ])
                .status()
                .map_err(|e| LynxError::StepFailed {
                    step: "registry".into(),
                    reason: format!("reg.exe failed: {e}"),
                })?;

            if !status.success() {
                return Err(LynxError::StepFailed {
                    step: "registry".into(),
                    reason: format!("reg.exe exited with: {status}"),
                });
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            self.sender.send(ProgressEvent::StepSkipped {
                step_index: index,
                step_label: label.to_string(),
                reason: "Registry steps are Windows-only".to_string(),
            });
            return Ok(());
        }

        self.sender.send(ProgressEvent::StepComplete {
            step_index: index,
            step_label: label.to_string(),
        });

        Ok(())
    }

    // ── Command ──────────────────────────────────────────────────────

    fn run_command(
        &self,
        index: usize,
        label: &str,
        command: &str,
        args: &[String],
        wait: bool,
        fail_on_error: bool,
        resolver: &TokenResolver,
    ) -> LynxResult<()> {
        self.sender.send(ProgressEvent::StepBegin {
            step_index: index,
            step_label: label.to_string(),
        });

        let resolved_cmd = resolver.resolve(command);
        let resolved_args: Vec<String> = args.iter().map(|a| resolver.resolve(a)).collect();

        let mut child = std::process::Command::new(&resolved_cmd)
            .args(&resolved_args)
            .spawn()
            .map_err(|e| LynxError::StepFailed {
                step: label.into(),
                reason: format!("Failed to spawn '{}': {e}", resolved_cmd),
            })?;

        if wait {
            let status = child.wait().map_err(|e| LynxError::StepFailed {
                step: label.into(),
                reason: format!("Command wait failed: {e}"),
            })?;

            if fail_on_error && !status.success() {
                return Err(LynxError::StepFailed {
                    step: label.into(),
                    reason: format!("Command exited with: {status}"),
                });
            }
        }

        self.sender.send(ProgressEvent::StepComplete {
            step_index: index,
            step_label: label.to_string(),
        });

        Ok(())
    }

    // ── EnvVar ───────────────────────────────────────────────────────

    fn run_env_var(
        &self,
        index: usize,
        label: &str,
        name: &str,
        value: &str,
        scope: &str,
        resolver: &TokenResolver,
    ) -> LynxResult<()> {
        self.sender.send(ProgressEvent::StepBegin {
            step_index: index,
            step_label: label.to_string(),
        });

        let resolved_value = resolver.resolve(value);

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;

            let reg_key = if scope == "system" {
                "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment"
            } else {
                "HKEY_CURRENT_USER\\Environment"
            };

            let status = Command::new("reg")
                .args(["add", reg_key, "/v", name, "/t", "REG_EXPAND_SZ", "/d", &resolved_value, "/f"])
                .status()
                .map_err(|e| LynxError::StepFailed {
                    step: "env_var".into(),
                    reason: format!("reg.exe failed: {e}"),
                })?;

            if !status.success() {
                return Err(LynxError::StepFailed {
                    step: "env_var".into(),
                    reason: "reg.exe failed to set environment variable".into(),
                });
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix, append to shell profile
            let profile = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".profile");

            let existing = std::fs::read_to_string(&profile).unwrap_or_default();
            let line = format!("\nexport {}=\"{}\"\n", name, resolved_value);
            std::fs::write(&profile, existing + &line)?;
        }

        self.sender.send(ProgressEvent::StepComplete {
            step_index: index,
            step_label: label.to_string(),
        });

        Ok(())
    }

    // ── Register Uninstaller ─────────────────────────────────────────

    fn run_register_uninstaller(
        &self,
        index: usize,
        label: &str,
        resolver: &TokenResolver,
    ) -> LynxResult<()> {
        self.sender.send(ProgressEvent::StepBegin {
            step_index: index,
            step_label: label.to_string(),
        });

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;

            let app = &self.project.app;
            let install_dir = resolver.resolve("{install_dir}");
            let uninstall_exe = format!("{}\\uninstall.exe", install_dir);

            let key = format!(
                "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}",
                app.id
            );

            let entries = [
                ("DisplayName",          app.name.as_str()),
                ("DisplayVersion",       app.version.as_str()),
                ("Publisher",            app.publisher.as_str()),
                ("InstallLocation",      &install_dir),
                ("UninstallString",      &uninstall_exe),
                ("NoModify",             "1"),
                ("NoRepair",             "1"),
            ];

            for (name, value) in &entries {
                let _status = Command::new("reg")
                    .args(["add", &key, "/v", name, "/t", "REG_SZ", "/d", value, "/f"])
                    .status();
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            self.sender.send(ProgressEvent::StepSkipped {
                step_index: index,
                step_label: label.to_string(),
                reason: "Uninstaller registration is Windows-only".to_string(),
            });
            return Ok(());
        }

        self.sender.send(ProgressEvent::StepComplete {
            step_index: index,
            step_label: label.to_string(),
        });

        Ok(())
    }
}
/// config.rs
///
/// Defines the full schema for a `.lynx` project file.
/// A `.lynx` file is TOML and describes everything about
/// an installer: app metadata, files to bundle, install steps,
/// and the UI theme to use.
///
/// Example .lynx file:
/// ```toml
/// [app]
/// name = "My App"
/// version = "1.0.0"
/// publisher = "Acme Corp"
/// id = "com.acme.myapp"
/// icon = "assets/icon.ico"
///
/// [theme]
/// name = "lynx-default"
/// path = "themes/lynx-default"
/// accent_color = "#FF6B35"
///
/// [[files]]
/// source = "dist/"
/// destination = "{install_dir}"
/// recursive = true
///
/// [[steps]]
/// kind = "extract"
/// label = "Installing files..."
///
/// [[steps]]
/// kind = "shortcut"
/// label = "Creating shortcuts..."
/// target = "{install_dir}/myapp.exe"
/// locations = ["desktop", "start_menu"]
/// ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{LynxResult, LynxError};

// ─────────────────────────────────────────────
//  Top-level project
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LynxProject {
    /// Core app metadata
    pub app: AppMeta,

    /// UI theme configuration
    pub theme: Theme,

    /// Files to bundle into the installer
    #[serde(default)]
    pub files: Vec<FileEntry>,

    /// Ordered list of install steps to execute
    #[serde(default)]
    pub steps: Vec<InstallStep>,

    /// Optional prerequisite checks before install begins
    #[serde(default)]
    pub prerequisites: Vec<Prerequisite>,

    /// Optional uninstaller config
    #[serde(default)]
    pub uninstall: Option<UninstallConfig>,
}

// ─────────────────────────────────────────────
//  App metadata
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMeta {
    /// Display name of the application
    pub name: String,

    /// Semantic version string e.g. "1.2.3"
    pub version: String,

    /// Publisher / company name
    pub publisher: String,

    /// Reverse-domain unique identifier e.g. "com.acme.myapp"
    pub id: String,

    /// Path to the app icon (relative to project root)
    #[serde(default)]
    pub icon: Option<String>,

    /// Default install directory token
    /// Supports tokens: {program_files}, {app_data}, {home}
    #[serde(default = "default_install_dir")]
    pub default_install_dir: String,

    /// URL to the app's website
    #[serde(default)]
    pub url: Option<String>,

    /// Short description shown in the installer
    #[serde(default)]
    pub description: Option<String>,
}

fn default_install_dir() -> String {
    "{program_files}/{app_name}".to_string()
}

// ─────────────────────────────────────────────
//  Theme
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name (must match a folder in themes/)
    pub name: String,

    /// Path to the theme folder (relative to project root)
    pub path: String,

    /// Primary accent color as hex string e.g. "#FF6B35"
    #[serde(default = "default_accent")]
    pub accent_color: String,

    /// Optional background color override
    #[serde(default)]
    pub background_color: Option<String>,

    /// Optional custom CSS variables to inject into the theme
    #[serde(default)]
    pub custom_vars: std::collections::HashMap<String, String>,
}

fn default_accent() -> String {
    "#FF6B35".to_string()
}

// ─────────────────────────────────────────────
//  Files
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Source path (relative to project root, or absolute)
    pub source: String,

    /// Destination path on the target machine
    /// Supports tokens: {install_dir}, {app_data}, {home}, {temp}
    pub destination: String,

    /// Whether to include subdirectories recursively
    #[serde(default = "default_true")]
    pub recursive: bool,

    /// Optional glob pattern to filter files e.g. "*.dll"
    #[serde(default)]
    pub filter: Option<String>,

    /// Whether to overwrite existing files
    #[serde(default = "default_true")]
    pub overwrite: bool,
}

fn default_true() -> bool { true }

// ─────────────────────────────────────────────
//  Install Steps
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstallStep {
    /// Extract bundled payload files to disk
    Extract {
        label: String,
    },

    /// Create a shortcut to an executable
    Shortcut {
        label: String,
        /// Path to the target executable (supports tokens)
        target: String,
        /// Where to place the shortcut
        locations: Vec<ShortcutLocation>,
        /// Optional custom shortcut name (defaults to app name)
        #[serde(default)]
        name: Option<String>,
    },

    /// Write a value to the Windows registry
    Registry {
        label: String,
        /// Registry hive e.g. "HKEY_LOCAL_MACHINE"
        hive: String,
        /// Registry key path
        key: String,
        /// Value name
        value_name: String,
        /// Value data
        value_data: String,
        /// Registry value type
        #[serde(default = "default_reg_type")]
        value_type: String,
    },

    /// Run an external command or script
    Command {
        label: String,
        /// Command to run (supports tokens)
        command: String,
        /// Arguments
        #[serde(default)]
        args: Vec<String>,
        /// Whether to wait for the command to finish
        #[serde(default = "default_true")]
        wait: bool,
        /// Whether a non-zero exit code should fail the install
        #[serde(default = "default_true")]
        fail_on_error: bool,
    },

    /// Set an environment variable
    EnvVar {
        label: String,
        name: String,
        value: String,
        /// "user" or "system"
        #[serde(default = "default_env_scope")]
        scope: String,
    },

    /// Register the app in Add/Remove Programs (Windows)
    RegisterUninstaller {
        label: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutLocation {
    Desktop,
    StartMenu,
    Taskbar,
}

fn default_reg_type() -> String { "REG_SZ".to_string() }
fn default_env_scope() -> String { "user".to_string() }

// ─────────────────────────────────────────────
//  Prerequisites
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Prerequisite {
    /// Check for a minimum .NET version
    DotNet {
        minimum_version: String,
        download_url: Option<String>,
    },
    /// Check for Visual C++ redistributable
    VcRedist {
        year: String,
        arch: String,
        download_url: Option<String>,
    },
    /// Check for a minimum OS version
    OsVersion {
        minimum: String,
    },
    /// Check for available disk space (in MB)
    DiskSpace {
        required_mb: u64,
    },
    /// Custom command-based check
    Custom {
        label: String,
        check_command: String,
        args: Vec<String>,
        /// Expected exit code to consider the check passed
        #[serde(default)]
        expected_exit_code: i32,
        error_message: String,
        download_url: Option<String>,
    },
}

// ─────────────────────────────────────────────
//  Uninstaller config
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallConfig {
    /// Whether to generate an uninstaller automatically
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Name of the uninstaller executable
    #[serde(default = "default_uninstaller_name")]
    pub exe_name: String,

    /// Additional files/dirs to remove on uninstall (beyond what was installed)
    #[serde(default)]
    pub extra_remove_paths: Vec<String>,

    /// Whether to remove the install directory itself
    #[serde(default = "default_true")]
    pub remove_install_dir: bool,
}

fn default_uninstaller_name() -> String { "uninstall.exe".to_string() }

// ─────────────────────────────────────────────
//  Parse helpers
// ─────────────────────────────────────────────

impl LynxProject {
    /// Parse a `.lynx` project from a TOML string
    pub fn from_toml_str(content: &str) -> LynxResult<Self> {
        toml::from_str(content).map_err(LynxError::TomlDe)
    }

    /// Load a `.lynx` project from a file path
    pub fn from_file(path: &PathBuf) -> LynxResult<Self> {
        if !path.exists() {
            return Err(LynxError::ConfigNotFound(
                path.to_string_lossy().to_string()
            ));
        }
        let content = std::fs::read_to_string(path)?;
        Self::from_toml_str(&content)
    }

    /// Serialize this project back to a TOML string
    pub fn to_toml_str(&self) -> LynxResult<String> {
        toml::to_string_pretty(self).map_err(|e| LynxError::TomlSer(e.to_string()))
    }
}
/// Lynx Engine — Core library for Lynx Installer
///
/// This crate is the foundation for everything Lynx does:
/// - Parsing `.lynx` project config files
/// - Bundling app payload into compressed archives
/// - Extracting and installing payloads on the target machine
/// - Reporting progress over IPC to the UI shell

pub mod config;
pub mod bundle;
pub mod extract;
pub mod error;
pub mod progress;
pub mod runner;

// Re-export the most important types at the crate root
pub use config::{LynxProject, AppMeta, InstallStep, FileEntry, Theme};
pub use error::LynxError;
pub use progress::{ProgressEvent, ProgressSender};
pub use runner::{InstallerRunner, RunnerConfig};
/// extract.rs
///
/// Responsible for decompressing and extracting a `.lynxpak` bundle
/// to the target machine during installation. Also handles token
/// resolution (replacing {install_dir}, {app_data}, etc.) in
/// destination paths.

use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::error::{LynxError, LynxResult};
use crate::progress::{ProgressEvent, ProgressSender};

// ─────────────────────────────────────────────
//  Token resolver
// ─────────────────────────────────────────────

/// Resolves path tokens like {install_dir} into real paths.
#[derive(Debug, Clone)]
pub struct TokenResolver {
    tokens: HashMap<String, String>,
}

impl TokenResolver {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    /// Set a token value
    pub fn set(&mut self, token: &str, value: &str) {
        self.tokens.insert(token.to_string(), value.to_string());
    }

    /// Build a resolver pre-populated with system paths for the current platform
    pub fn with_defaults(app_name: &str, install_dir: &str) -> Self {
        let mut r = Self::new();

        r.set("install_dir", install_dir);
        r.set("app_name", app_name);

        // Platform-specific defaults
        #[cfg(target_os = "windows")]
        {
            let pf = std::env::var("ProgramFiles")
                .unwrap_or_else(|_| "C:\\Program Files".to_string());
            let appdata = std::env::var("APPDATA")
                .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".to_string());
            let localappdata = std::env::var("LOCALAPPDATA")
                .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".to_string());
            let temp = std::env::var("TEMP")
                .unwrap_or_else(|_| "C:\\Windows\\Temp".to_string());
            r.set("program_files", &pf);
            r.set("app_data", &appdata);
            r.set("local_app_data", &localappdata);
            r.set("temp", &temp);
        }

        #[cfg(target_os = "macos")]
        {
            let home = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/Users/user"))
                .to_string_lossy()
                .to_string();
            r.set("home", &home);
            r.set("app_data", &format!("{}/Library/Application Support/{}", home, app_name));
            r.set("temp", "/tmp");
            r.set("applications", "/Applications");
        }

        #[cfg(target_os = "linux")]
        {
            let home = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/home/user"))
                .to_string_lossy()
                .to_string();
            r.set("home", &home);
            r.set("app_data", &format!("{}/.config/{}", home, app_name));
            r.set("temp", "/tmp");
        }

        r
    }

    /// Replace all {token} occurrences in a path string
    pub fn resolve(&self, input: &str) -> String {
        let mut result = input.to_string();
        for (token, value) in &self.tokens {
            result = result.replace(&format!("{{{token}}}"), value);
        }
        result
    }
}

impl Default for TokenResolver {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────
//  Extractor
// ─────────────────────────────────────────────

pub struct Extractor<'a> {
    sender: &'a ProgressSender,
    resolver: TokenResolver,
    step_index: usize,
}

impl<'a> Extractor<'a> {
    pub fn new(sender: &'a ProgressSender, resolver: TokenResolver, step_index: usize) -> Self {
        Self { sender, resolver, step_index }
    }

    /// Extract a `.lynxpak` archive to disk.
    ///
    /// `pak_path` — path to the .lynxpak file (or a bytes slice embedded in the binary)
    pub fn extract_from_file(&self, pak_path: &Path) -> LynxResult<Vec<PathBuf>> {
        let file = fs::File::open(pak_path)
            .map_err(|e| LynxError::Extract(format!("Cannot open bundle: {e}")))?;

        let buf = BufReader::new(file);
        let decoder = zstd::stream::Decoder::new(buf)
            .map_err(|e| LynxError::Extract(format!("Cannot init zstd decoder: {e}")))?;

        self.extract_from_reader(decoder)
    }

    /// Extract from a raw byte slice (for when the pak is embedded in the exe).
    pub fn extract_from_bytes(&self, data: &[u8]) -> LynxResult<Vec<PathBuf>> {
        let cursor = std::io::Cursor::new(data);
        let decoder = zstd::stream::Decoder::new(cursor)
            .map_err(|e| LynxError::Extract(format!("Cannot init zstd decoder: {e}")))?;
        self.extract_from_reader(decoder)
    }

    fn extract_from_reader<R: std::io::Read>(
        &self,
        reader: R,
    ) -> LynxResult<Vec<PathBuf>> {
        let mut archive = tar::Archive::new(reader);
        let mut extracted: Vec<PathBuf> = Vec::new();

        self.sender.send(ProgressEvent::StepBegin {
            step_index: self.step_index,
            step_label: "Extracting files...".to_string(),
        });

        let entries = archive
            .entries()
            .map_err(|e| LynxError::Extract(format!("Cannot read archive entries: {e}")))?;

        for entry_result in entries {
            let mut entry = entry_result
                .map_err(|e| LynxError::Extract(format!("Bad archive entry: {e}")))?;

            let archive_path = entry
                .path()
                .map_err(|e| LynxError::Extract(e.to_string()))?
                .to_string_lossy()
                .to_string();

            // Resolve destination using token system
            let dest_str = self.resolver.resolve(&archive_path);
            let dest_path = PathBuf::from(&dest_str);

            // Create parent directories as needed
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| LynxError::Extract(format!(
                        "Cannot create dir {}: {e}",
                        parent.display()
                    )))?;
            }

            // Extract the file
            entry.unpack(&dest_path)
                .map_err(|e| LynxError::Extract(format!(
                    "Cannot extract {}: {e}",
                    dest_path.display()
                )))?;

            let file_name = dest_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            self.sender.send(ProgressEvent::FileProgress {
                step_index: self.step_index,
                file_name,
                fraction: 0.0, // Will be updated with manifest data in future
                bytes_written: 0,
                bytes_total: 0,
            });

            extracted.push(dest_path);
        }

        self.sender.send(ProgressEvent::StepComplete {
            step_index: self.step_index,
            step_label: format!("Extracted {} files", extracted.len()),
        });

        Ok(extracted)
    }
}

// ─────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_resolution() {
        let mut resolver = TokenResolver::new();
        resolver.set("install_dir", "C:\\Program Files\\MyApp");
        resolver.set("app_name", "MyApp");

        let resolved = resolver.resolve("{install_dir}\\{app_name}.exe");
        assert_eq!(resolved, "C:\\Program Files\\MyApp\\MyApp.exe");
    }

    #[test]
    fn test_token_resolution_unknown_token() {
        let resolver = TokenResolver::new();
        // Unknown tokens are left as-is
        let resolved = resolver.resolve("{unknown_token}/file.txt");
        assert_eq!(resolved, "{unknown_token}/file.txt");
    }

    #[test]
    fn test_token_resolution_multiple() {
        let mut resolver = TokenResolver::new();
        resolver.set("install_dir", "/opt/myapp");
        resolver.set("app_name", "myapp");

        let resolved = resolver.resolve("{install_dir}/bin/{app_name}");
        assert_eq!(resolved, "/opt/myapp/bin/myapp");
    }
}
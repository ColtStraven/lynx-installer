/// bundle.rs
///
/// Responsible for taking a LynxProject's file entries and bundling
/// them into a compressed `.lynxpak` archive that gets embedded
/// into the final installer executable.
///
/// Format: tar + zstd compression
/// A `.lynxpak` is just a zstd-compressed tar archive with a
/// manifest.json at the root describing all included files.

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};

use crate::config::{FileEntry, LynxProject};
use crate::error::{LynxError, LynxResult};
use crate::progress::{ProgressEvent, ProgressSender};

// ─────────────────────────────────────────────
//  Manifest (stored inside the .lynxpak)
// ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleManifest {
    /// Lynx engine version that created this bundle
    pub lynx_version: String,
    /// App name
    pub app_name: String,
    /// App version
    pub app_version: String,
    /// Total number of files in the bundle
    pub file_count: usize,
    /// Total uncompressed size in bytes
    pub total_bytes: u64,
    /// All file entries with their destination tokens
    pub entries: Vec<BundleEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleEntry {
    /// Path inside the tar archive
    pub archive_path: String,
    /// Destination token path e.g. "{install_dir}/myapp.exe"
    pub destination: String,
    /// File size in bytes
    pub size: u64,
}

// ─────────────────────────────────────────────
//  Bundler
// ─────────────────────────────────────────────

pub struct Bundler<'a> {
    project: &'a LynxProject,
    project_root: PathBuf,
    sender: &'a ProgressSender,
}

impl<'a> Bundler<'a> {
    pub fn new(
        project: &'a LynxProject,
        project_root: PathBuf,
        sender: &'a ProgressSender,
    ) -> Self {
        Self { project, project_root, sender }
    }

    /// Bundle all files from the project into a `.lynxpak` at the given output path.
    pub fn bundle(&self, output_path: &Path) -> LynxResult<BundleManifest> {
        // Collect all files to bundle
        let collected = self.collect_files()?;

        let total_bytes: u64 = collected.iter().map(|(_, _, size)| *size).sum();
        let file_count = collected.len();

        self.sender.send(ProgressEvent::StepBegin {
            step_index: 0,
            step_label: format!("Bundling {} files ({} KB)...", file_count, total_bytes / 1024),
        });

        // Create the output file
        let out_file = File::create(output_path)
            .map_err(|e| LynxError::Bundle(format!("Cannot create output file: {e}")))?;
        let buf_writer = BufWriter::new(out_file);

        // Wrap in zstd encoder
        let zstd_encoder = zstd::stream::Encoder::new(buf_writer, 3)
            .map_err(|e| LynxError::Compression(e.to_string()))?;

        // Wrap in tar builder
        let mut tar = tar::Builder::new(zstd_encoder);

        let mut entries: Vec<BundleEntry> = Vec::new();
        let mut bytes_written: u64 = 0;

        for (source_path, destination_token, size) in &collected {
            let archive_path = self.make_archive_path(source_path);

            // Append the file to the tar
            tar.append_path_with_name(source_path, &archive_path)
                .map_err(|e| LynxError::Bundle(format!("Failed to add {}: {e}", archive_path)))?;

            bytes_written += size;

            let fraction = bytes_written as f32 / total_bytes.max(1) as f32;
            let file_name = source_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            self.sender.send(ProgressEvent::FileProgress {
                step_index: 0,
                file_name,
                fraction,
                bytes_written,
                bytes_total: total_bytes,
            });

            entries.push(BundleEntry {
                archive_path,
                destination: destination_token.clone(),
                size: *size,
            });
        }

        // Finalize the tar + zstd stream
        let zstd_encoder = tar.into_inner()
            .map_err(|e| LynxError::Bundle(format!("Failed to finalize tar: {e}")))?;
        zstd_encoder.finish()
            .map_err(|e| LynxError::Compression(e.to_string()))?;

        let manifest = BundleManifest {
            lynx_version: env!("CARGO_PKG_VERSION").to_string(),
            app_name: self.project.app.name.clone(),
            app_version: self.project.app.version.clone(),
            file_count,
            total_bytes,
            entries,
        };

        self.sender.send(ProgressEvent::StepComplete {
            step_index: 0,
            step_label: "Bundle complete".to_string(),
        });

        Ok(manifest)
    }

    // ── Private helpers ──────────────────────────────────────────────

    /// Walk all FileEntry items and collect (source_path, destination_token, size) tuples
    fn collect_files(&self) -> LynxResult<Vec<(PathBuf, String, u64)>> {
        let mut collected: Vec<(PathBuf, String, u64)> = Vec::new();

        for entry in &self.project.files {
            let source_path = self.resolve_source(&entry.source);

            if source_path.is_file() {
                let size = source_path.metadata()?.len();
                collected.push((source_path, entry.destination.clone(), size));
            } else if source_path.is_dir() {
                self.walk_dir(&source_path, entry, &mut collected)?;
            } else {
                return Err(LynxError::Bundle(format!(
                    "Source path does not exist: {}",
                    source_path.display()
                )));
            }
        }

        Ok(collected)
    }

    fn walk_dir(
        &self,
        dir: &Path,
        entry: &FileEntry,
        collected: &mut Vec<(PathBuf, String, u64)>,
    ) -> LynxResult<()> {
        let walker = if entry.recursive {
            WalkDir::new(dir)
        } else {
            WalkDir::new(dir).max_depth(1)
        };

        for result in walker {
            let item = result.map_err(|e| LynxError::Bundle(e.to_string()))?;
            let path = item.path().to_path_buf();

            if !path.is_file() {
                continue;
            }

            // Apply glob filter if specified
            if let Some(filter) = &entry.filter {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if !glob_match(filter, &file_name) {
                    continue;
                }
            }

            let size = path.metadata()?.len();

            // Build destination token preserving relative sub-path
            let rel = path.strip_prefix(dir).unwrap_or(&path);
            let dest = format!("{}/{}", entry.destination, rel.to_string_lossy().replace('\\', "/"));

            collected.push((path, dest, size));
        }

        Ok(())
    }

    fn resolve_source(&self, source: &str) -> PathBuf {
        let p = PathBuf::from(source);
        if p.is_absolute() {
            p
        } else {
            self.project_root.join(p)
        }
    }

    fn make_archive_path(&self, source: &Path) -> String {
        // Strip the project root prefix to get a clean relative path
        source
            .strip_prefix(&self.project_root)
            .unwrap_or(source)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

// ─────────────────────────────────────────────
//  Simple glob matcher (supports * wildcard only for now)
// ─────────────────────────────────────────────

fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(ext) = pattern.strip_prefix("*.") {
        return text.ends_with(&format!(".{ext}"));
    }
    pattern == text
}

// ─────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match_wildcard() {
        assert!(glob_match("*", "anything.exe"));
        assert!(glob_match("*.dll", "mylib.dll"));
        assert!(!glob_match("*.dll", "mylib.exe"));
        assert!(glob_match("*.exe", "app.exe"));
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("myapp.exe", "myapp.exe"));
        assert!(!glob_match("myapp.exe", "otherapp.exe"));
    }
}
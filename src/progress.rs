/// progress.rs
///
/// Defines the progress event system used by the engine to
/// communicate install status to the UI shell in real time.
///
/// Events are serialized as JSON and sent over the IPC channel
/// (named pipe or stdout depending on the context).

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────
//  Event types
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEvent {
    /// Install process is starting
    Started {
        app_name: String,
        app_version: String,
        total_steps: usize,
    },

    /// A specific install step is beginning
    StepBegin {
        step_index: usize,
        step_label: String,
    },

    /// File extraction progress (0.0 to 1.0)
    FileProgress {
        step_index: usize,
        /// Current file being extracted
        file_name: String,
        /// 0.0 to 1.0
        fraction: f32,
        /// Bytes written so far
        bytes_written: u64,
        /// Total bytes to write
        bytes_total: u64,
    },

    /// A step completed successfully
    StepComplete {
        step_index: usize,
        step_label: String,
    },

    /// A step was skipped (e.g. prerequisite already satisfied)
    StepSkipped {
        step_index: usize,
        step_label: String,
        reason: String,
    },

    /// A non-fatal warning occurred
    Warning {
        message: String,
    },

    /// Install completed successfully
    Complete {
        install_dir: String,
        duration_ms: u64,
    },

    /// Install failed
    Failed {
        step_index: Option<usize>,
        step_label: Option<String>,
        error: String,
    },

    /// Uninstall progress (mirrors install but simpler)
    UninstallProgress {
        fraction: f32,
        current_action: String,
    },

    /// Uninstall complete
    UninstallComplete,
}

impl ProgressEvent {
    /// Serialize to a JSON string for IPC transport
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"type":"warning","message":"Failed to serialize progress event"}"#.to_string()
        })
    }

    /// Deserialize from a JSON string
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

// ─────────────────────────────────────────────
//  Sender abstraction
// ─────────────────────────────────────────────

/// A simple callback-based sender. The UI shell registers a closure
/// that receives events. In the real IPC implementation this will
/// write to a named pipe — but this abstraction keeps the engine
/// decoupled from the transport layer.
pub struct ProgressSender {
    callback: Box<dyn Fn(ProgressEvent) + Send + Sync>,
}

impl ProgressSender {
    /// Create a new sender with a callback closure
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(ProgressEvent) + Send + Sync + 'static,
    {
        Self {
            callback: Box::new(callback),
        }
    }

    /// Create a no-op sender for testing or headless use
    pub fn noop() -> Self {
        Self::new(|_| {})
    }

    /// Create a sender that prints JSON events to stdout
    pub fn stdout() -> Self {
        Self::new(|event| {
            println!("{}", event.to_json());
        })
    }

    /// Send a progress event
    pub fn send(&self, event: ProgressEvent) {
        (self.callback)(event);
    }
}

impl std::fmt::Debug for ProgressSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressSender").finish()
    }
}
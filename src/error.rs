use thiserror::Error;

#[derive(Debug, Error)]
pub enum LynxError {
    #[error("Config parse error: {0}")]
    ConfigParse(String),

    #[error("Config file not found at path: {0}")]
    ConfigNotFound(String),

    #[error("Bundle error: {0}")]
    Bundle(String),

    #[error("Extraction error: {0}")]
    Extract(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// toml 0.9 ser::Error is a simple string type, not a full std::error::Error impl
    #[error("TOML serialization error: {0}")]
    TomlSer(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Install step failed [{step}]: {reason}")]
    StepFailed { step: String, reason: String },

    #[error("Unsupported platform for operation: {0}")]
    UnsupportedPlatform(String),
}

impl From<toml::ser::Error> for LynxError {
    fn from(e: toml::ser::Error) -> Self {
        LynxError::TomlSer(e.to_string())
    }
}

pub type LynxResult<T> = Result<T, LynxError>;
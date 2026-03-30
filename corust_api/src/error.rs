//! Error types shared across the CLI ↔ core boundary.

use std::io;

/// Errors that can occur across the public API boundary.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// The underlying communication channel was closed unexpectedly.
    #[error("channel closed")]
    ChannelClosed,

    /// An I/O error occurred (e.g., loading configuration files).
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization / deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration loading or validation failed.
    #[error("configuration error: {0}")]
    Config(String),

    /// The agent engine returned an error.
    #[error("agent error: {0}")]
    Agent(String),
}

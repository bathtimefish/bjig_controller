//! Error types for bjig_controller

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for bjig_controller operations
pub type Result<T> = std::result::Result<T, BjigError>;

/// Error types for bjig_controller operations
#[derive(Debug, Error)]
pub enum BjigError {
    /// Bjig binary not found at the specified path
    #[error("Bjig binary not found: {0}")]
    BinaryNotFound(PathBuf),

    /// Command execution failed with error message
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    /// Failed to parse JSON output from bjig command
    #[error("Failed to parse JSON output: {0}")]
    JsonParseError(#[from] serde_json::Error),

    /// IO error occurred during command execution
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serial port not configured
    #[error("Serial port not configured. Use .with_port() or set BJIG_CLI_PORT environment variable")]
    PortNotConfigured,

    /// Baud rate not configured
    #[error("Baud rate not configured. Use .with_baud() or set BJIG_CLI_BAUD environment variable")]
    BaudNotConfigured,

    /// UTF-8 decode error
    #[error("UTF-8 decode error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    /// Invalid parameter provided
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
}

//! Spec-Kit Error Types

use thiserror::Error;

/// Errors that can occur when interacting with spec-kit CLI
#[derive(Error, Debug)]
pub enum SpecKitError {
    #[error("Spec-kit CLI not found. Please install with: uv tool install specify-cli")]
    CliNotFound,

    #[error("Python 3.11+ required but not found")]
    PythonVersionTooOld,

    #[error("Spec-kit command failed: {command}\nStderr: {stderr}\nExit code: {exit_code}")]
    CommandFailed {
        command: String,
        stderr: String,
        exit_code: i32,
    },

    #[error("Failed to parse spec-kit output: {0}")]
    ParseError(String),

    #[error("Timeout waiting for spec-kit command")]
    Timeout,

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Project not initialized: {0}")]
    ProjectNotInitialized(String),

    #[error("Invalid file path: {path}. Reason: {reason}")]
    InvalidFilePath { path: String, reason: String },

    #[error("Template processing failed: {0}")]
    TemplateError(String),

    #[error("File operation failed: {operation} on {path}")]
    FileOperationError { operation: String, path: String },

    #[error("Input validation failed: {0}")]
    ValidationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Content too large: {size} bytes exceeds limit of {limit} bytes")]
    ContentTooLarge { size: usize, limit: usize },

    #[error("Invalid encoding: {0}")]
    EncodingError(String),
}

impl SpecKitError {
    /// Create a command failed error
    pub fn command_failed(
        command: impl Into<String>,
        stderr: impl Into<String>,
        exit_code: i32,
    ) -> Self {
        Self::CommandFailed {
            command: command.into(),
            stderr: stderr.into(),
            exit_code,
        }
    }

    /// Get recovery suggestion for this error
    pub fn recovery_suggestion(&self) -> String {
        match self {
            Self::ProjectNotInitialized(_) => {
                "Run 'speckit_init' to initialize the project structure".to_string()
            }
            Self::InvalidPath(_) | Self::InvalidFilePath { .. } => {
                "Ensure the path is within the project directory and does not contain '..'"
                    .to_string()
            }
            Self::CliNotFound => {
                "Install uv/uvx with: curl -LsSf https://astral.sh/uv/install.sh | sh".to_string()
            }
            Self::Timeout => {
                "Try increasing the timeout with --timeout flag or check system resources"
                    .to_string()
            }
            Self::ValidationError(_) => {
                "Check input parameters and ensure they meet the requirements".to_string()
            }
            Self::ContentTooLarge { limit, .. } => {
                format!("Reduce content size to under {} bytes", limit)
            }
            Self::TemplateError(_) => {
                "Verify template files exist in .specify/templates/ directory".to_string()
            }
            Self::FileOperationError { .. } => {
                "Check file permissions and ensure the path is accessible".to_string()
            }
            Self::ConfigError(_) => {
                "Review configuration file syntax and required fields".to_string()
            }
            Self::EncodingError(_) => "Ensure content is valid UTF-8 encoded text".to_string(),
            _ => "Check the error message for details".to_string(),
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Timeout | Self::IoError(_) | Self::CommandFailed { .. }
        )
    }
}

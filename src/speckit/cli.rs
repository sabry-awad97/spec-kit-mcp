//! Spec-Kit CLI Executor
//!
//! Handles spawning and managing spec-kit CLI processes.

use anyhow::{Context, Result};
use async_process::{Command, Stdio};
use std::path::Path;
use std::time::Duration;
use tokio::time::timeout;

use super::errors::SpecKitError;

/// Result of executing a spec-kit command
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Exit code
    pub exit_code: i32,
}

impl CommandResult {
    /// Check if the command was successful
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// Get a summary of the result
    pub fn summary(&self) -> String {
        if self.is_success() {
            format!("Success\n{}", self.stdout.trim())
        } else {
            format!(
                "Failed (exit code {})\n{}",
                self.exit_code,
                self.stderr.trim()
            )
        }
    }
}

/// Spec-kit CLI integration
#[derive(Debug, Clone)]
pub struct SpecKitCli {
    /// Path to the specify command
    cli_path: String,

    /// Path to Python interpreter (reserved for future use)
    #[allow(dead_code)]
    python_path: String,

    /// Default timeout for commands (in seconds)
    timeout_seconds: u64,

    /// Test mode flag
    test_mode: bool,
}

impl SpecKitCli {
    /// Create a new spec-kit CLI interface
    pub fn new() -> Self {
        Self {
            cli_path: "uvx".to_string(),
            python_path: "python3".to_string(),
            timeout_seconds: 300, // 5 minutes
            test_mode: false,
        }
    }

    /// Create a test mode CLI (returns mock data)
    #[cfg(test)]
    pub fn new_test_mode() -> Self {
        Self {
            cli_path: "specify".to_string(),
            python_path: "python3".to_string(),
            timeout_seconds: 300,
            test_mode: true,
        }
    }

    /// Set the CLI path
    pub fn with_cli_path(mut self, path: impl Into<String>) -> Self {
        self.cli_path = path.into();
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Check if spec-kit is installed (via uvx)
    pub async fn is_installed(&self) -> bool {
        if self.test_mode {
            return true;
        }

        // Check if uvx is available - that's all we need since uvx will
        // automatically download spec-kit from GitHub when needed
        let uvx_available = Command::new(&self.cli_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false);

        if !uvx_available {
            tracing::warn!("uvx command not found - spec-kit requires uv/uvx");
            return false;
        }

        tracing::info!("uvx is available - spec-kit will be run via uvx");
        true
    }

    /// Execute a spec-kit command
    async fn execute_command(&self, args: &[&str]) -> Result<CommandResult> {
        if self.test_mode {
            return Ok(CommandResult {
                stdout: "Test mode: command executed successfully".to_string(),
                stderr: String::new(),
                exit_code: 0,
            });
        }

        // Build the full command with uvx + spec-kit repo + specify + args
        let mut full_args = vec![
            "--from",
            "git+https://github.com/github/spec-kit.git",
            "specify",
        ];
        full_args.extend_from_slice(args);

        tracing::debug!(
            command = %self.cli_path,
            args = ?full_args,
            "Executing spec-kit command via uvx"
        );

        let command_future = Command::new(&self.cli_path)
            .args(&full_args)
            .env("PYTHONIOENCODING", "utf-8") // Force UTF-8 encoding for Windows
            .env("PYTHONUTF8", "1") // Enable UTF-8 mode (Python 3.7+)
            .stdin(Stdio::null()) // Prevent interactive prompts
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        let output = timeout(Duration::from_secs(self.timeout_seconds), command_future)
            .await
            .context("Command timeout")?
            .context("Failed to execute command")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        let result = CommandResult {
            stdout,
            stderr,
            exit_code,
        };

        if !result.is_success() {
            tracing::warn!(
                exit_code = result.exit_code,
                stderr = %result.stderr,
                "Spec-kit command failed"
            );
        }

        Ok(result)
    }

    /// Initialize a new spec-kit project
    pub async fn init(&self, project_name: &str, path: &Path) -> Result<CommandResult> {
        // Determine if we're initializing in current directory or creating new
        let is_current_dir = path == Path::new(".") || path == Path::new("");

        let mut args = vec!["init"];

        if is_current_dir {
            // Initialize in current directory
            args.push(".");
        } else {
            // Create new project directory
            args.push(project_name);
        }

        // Add --ai flag with a default value to avoid interactive prompts
        args.push("--ai");
        args.push("claude");

        // Add --ignore-agent-tools to skip interactive prompts
        args.push("--ignore-agent-tools");

        // Add --no-git to avoid git-related prompts
        args.push("--no-git");

        let result = self.execute_command(&args).await?;

        if !result.is_success() {
            return Err(SpecKitError::command_failed(
                format!("specify init {}", project_name),
                &result.stderr,
                result.exit_code,
            )
            .into());
        }

        Ok(result)
    }

    /// Check for installed tools
    pub async fn check(&self) -> Result<CommandResult> {
        let result = self.execute_command(&["check"]).await?;

        if !result.is_success() {
            return Err(SpecKitError::command_failed(
                "specify check",
                &result.stderr,
                result.exit_code,
            )
            .into());
        }

        Ok(result)
    }
}

impl Default for SpecKitCli {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cli_creation() {
        let cli = SpecKitCli::new();
        assert_eq!(cli.timeout_seconds, 300);
    }

    #[tokio::test]
    async fn test_init() {
        let cli = SpecKitCli::new_test_mode();
        let dir = tempdir().unwrap();

        let result = cli.init("test-project", dir.path()).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_check() {
        let cli = SpecKitCli::new_test_mode();
        let result = cli.check().await.unwrap();
        assert!(result.is_success());
    }
}

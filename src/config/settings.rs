//! Configuration Settings
//!
//! Centralized configuration management with environment variable support.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SpecKitConfig {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,

    /// Tool configuration
    #[serde(default)]
    pub tools: ToolsConfig,

    /// Resource limits
    #[serde(default)]
    pub limits: LimitsConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Server host (for future HTTP support)
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port (for future HTTP support)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Enable stdio transport (default: true)
    #[serde(default = "default_true")]
    pub stdio_enabled: bool,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_true() -> bool {
    true
}

/// Tools configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolsConfig {
    /// Maximum retries for failed operations
    #[serde(default = "default_retries")]
    pub max_retries: u32,
}

fn default_retries() -> u32 {
    3
}

/// Resource limits configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    /// Maximum file size in bytes
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,

    /// Maximum content length in bytes
    #[serde(default = "default_max_content_length")]
    pub max_content_length: usize,

    /// Maximum concurrent operations
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_operations: usize,

    /// Maximum path length
    #[serde(default = "default_max_path_length")]
    pub max_path_length: usize,
}

fn default_max_file_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_max_content_length() -> usize {
    1024 * 1024 // 1MB
}

fn default_max_concurrent() -> usize {
    10
}

fn default_max_path_length() -> usize {
    4096
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format (text, json)
    #[serde(default = "default_log_format")]
    pub format: String,

    /// Log output (stdout, stderr, file)
    #[serde(default = "default_log_output")]
    pub output: String,

    /// Log file path (if output is "file")
    pub file_path: Option<PathBuf>,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "text".to_string()
}

fn default_log_output() -> String {
    "stderr".to_string()
}

/// Security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    /// Enable authentication (for future HTTP support)
    #[serde(default)]
    pub enable_auth: bool,

    /// API keys (for future HTTP support)
    #[serde(default)]
    pub api_keys: Vec<String>,

    /// Enable rate limiting
    #[serde(default)]
    pub enable_rate_limiting: bool,

    /// Rate limit: max requests per window
    #[serde(default = "default_rate_limit")]
    pub rate_limit_requests: usize,

    /// Rate limit window in seconds
    #[serde(default = "default_rate_window")]
    pub rate_limit_window_seconds: u64,

    /// Enable audit logging
    #[serde(default)]
    pub enable_audit_log: bool,

    /// Audit log file path
    pub audit_log_path: Option<PathBuf>,
}

fn default_rate_limit() -> usize {
    100
}

fn default_rate_window() -> u64 {
    60
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            stdio_enabled: default_true(),
        }
    }
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            max_retries: default_retries(),
        }
    }
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_file_size: default_max_file_size(),
            max_content_length: default_max_content_length(),
            max_concurrent_operations: default_max_concurrent(),
            max_path_length: default_max_path_length(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            output: default_log_output(),
            file_path: None,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_auth: false,
            api_keys: Vec::new(),
            enable_rate_limiting: false,
            rate_limit_requests: default_rate_limit(),
            rate_limit_window_seconds: default_rate_window(),
            enable_audit_log: false,
            audit_log_path: None,
        }
    }
}

impl SpecKitConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Override with environment variables
        if let Ok(level) = std::env::var("SPECKIT_LOG_LEVEL") {
            config.logging.level = level;
        }

        if let Ok(max_size) = std::env::var("SPECKIT_MAX_FILE_SIZE") {
            config.limits.max_file_size = max_size
                .parse()
                .context("Invalid SPECKIT_MAX_FILE_SIZE value")?;
        }

        Ok(config)
    }

    /// Load configuration from TOML file
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Load configuration with precedence: file -> env -> defaults
    pub fn load() -> Result<Self> {
        // Start with defaults
        let mut config = Self::default();

        // Try to load from file
        let mut config_paths = vec![
            PathBuf::from("speckit.toml"),
            PathBuf::from("config/speckit.toml"),
        ];

        // Add user config directory if available
        if let Some(config_dir) = dirs::config_dir() {
            config_paths.push(config_dir.join("speckit/config.toml"));
        }

        for path in config_paths {
            if path.exists() {
                tracing::info!(path = %path.display(), "Loading configuration from file");
                config = Self::from_file(&path)?;
                break;
            }
        }

        // Override with environment variables
        if let Ok(level) = std::env::var("SPECKIT_LOG_LEVEL") {
            config.logging.level = level;
        }

        if let Ok(max_size) = std::env::var("SPECKIT_MAX_FILE_SIZE") {
            config.limits.max_file_size = max_size
                .parse()
                .context("Invalid SPECKIT_MAX_FILE_SIZE value")?;
        }

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(anyhow::anyhow!(
                "Invalid log level: {}. Must be one of: {}",
                self.logging.level,
                valid_levels.join(", ")
            ));
        }

        // Validate log format
        let valid_formats = ["text", "json"];
        if !valid_formats.contains(&self.logging.format.as_str()) {
            return Err(anyhow::anyhow!(
                "Invalid log format: {}. Must be one of: {}",
                self.logging.format,
                valid_formats.join(", ")
            ));
        }

        // Validate limits
        if self.limits.max_file_size == 0 {
            return Err(anyhow::anyhow!("max_file_size must be greater than 0"));
        }

        if self.limits.max_content_length == 0 {
            return Err(anyhow::anyhow!("max_content_length must be greater than 0"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SpecKitConfig::default();
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.limits.max_file_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_config_validation() {
        let config = SpecKitConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = SpecKitConfig::default();
        invalid_config.logging.level = "invalid".to_string();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_from_env() {
        std::env::set_var("SPECKIT_LOG_LEVEL", "debug");

        let config = SpecKitConfig::from_env().unwrap();
        assert_eq!(config.logging.level, "debug");

        std::env::remove_var("SPECKIT_LOG_LEVEL");
    }
}

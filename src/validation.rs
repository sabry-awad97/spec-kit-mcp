//! Input Validation & Sanitization
//!
//! Comprehensive validation for all user inputs to prevent security issues
//! and ensure data integrity.

use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::Path;

/// Configuration for input validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum file size in bytes (default: 10MB)
    pub max_file_size: usize,

    /// Maximum content length in bytes (default: 1MB)
    pub max_content_length: usize,

    /// Maximum path length (default: 4096)
    pub max_path_length: usize,

    /// Allow unicode in content (default: true)
    pub allow_unicode: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_content_length: 1024 * 1024, // 1MB
            max_path_length: 4096,
            allow_unicode: true,
        }
    }
}

/// Input validator with configurable limits
#[derive(Clone)]
pub struct InputValidator {
    config: ValidationConfig,
    filename_regex: Regex,
}

impl InputValidator {
    /// Create a new validator with default configuration
    pub fn new() -> Self {
        Self::with_config(ValidationConfig::default())
    }

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        // Regex for safe filenames: alphanumeric, dash, underscore, dot
        let filename_regex = Regex::new(r"^[a-zA-Z0-9_\-\.]+$").unwrap();

        Self {
            config,
            filename_regex,
        }
    }

    /// Validate requirements text
    pub fn validate_requirements(&self, input: &str) -> Result<()> {
        // Check length
        if input.is_empty() {
            return Err(anyhow!("Requirements cannot be empty"));
        }

        if input.len() > self.config.max_content_length {
            return Err(anyhow!(
                "Requirements too long: {} bytes exceeds limit of {} bytes",
                input.len(),
                self.config.max_content_length
            ));
        }

        // Check for path traversal in content
        if input.contains("../") || input.contains("..\\") {
            return Err(anyhow!(
                "Content contains potential path traversal sequences"
            ));
        }

        // Validate encoding
        self.validate_encoding(input)?;

        Ok(())
    }

    /// Validate user stories text
    pub fn validate_user_stories(&self, input: &str) -> Result<()> {
        if input.is_empty() {
            return Ok(()); // User stories are optional
        }

        if input.len() > self.config.max_content_length {
            return Err(anyhow!(
                "User stories too long: {} bytes exceeds limit of {} bytes",
                input.len(),
                self.config.max_content_length
            ));
        }

        self.validate_encoding(input)?;

        Ok(())
    }

    /// Validate principles text
    pub fn validate_principles(&self, input: &str) -> Result<()> {
        if input.is_empty() {
            return Err(anyhow!("Principles cannot be empty"));
        }

        if input.len() > self.config.max_content_length {
            return Err(anyhow!(
                "Principles too long: {} bytes exceeds limit of {} bytes",
                input.len(),
                self.config.max_content_length
            ));
        }

        self.validate_encoding(input)?;

        Ok(())
    }

    /// Validate project name
    pub fn validate_project_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(anyhow!("Project name cannot be empty"));
        }

        if name.len() > 255 {
            return Err(anyhow!("Project name too long (max 255 characters)"));
        }

        // Check for invalid characters
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(anyhow!(
                "Project name can only contain alphanumeric characters, hyphens, and underscores"
            ));
        }

        // Check for reserved names
        let reserved = [".", "..", "con", "prn", "aux", "nul"];
        if reserved.contains(&name.to_lowercase().as_str()) {
            return Err(anyhow!("Project name '{}' is reserved", name));
        }

        Ok(())
    }

    /// Validate and sanitize filename
    pub fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
            .collect()
    }

    /// Validate filename
    pub fn validate_filename(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(anyhow!("Filename cannot be empty"));
        }

        if name.len() > 255 {
            return Err(anyhow!("Filename too long (max 255 characters)"));
        }

        if !self.filename_regex.is_match(name) {
            return Err(anyhow!(
                "Filename contains invalid characters. Only alphanumeric, dash, underscore, and dot allowed"
            ));
        }

        // Check for reserved names (Windows)
        let name_lower = name.to_lowercase();
        let reserved = ["con", "prn", "aux", "nul", "com1", "com2", "lpt1", "lpt2"];
        for r in &reserved {
            if name_lower == *r || name_lower.starts_with(&format!("{}.", r)) {
                return Err(anyhow!("Filename '{}' is reserved", name));
            }
        }

        Ok(())
    }

    /// Validate path length
    pub fn validate_path_length(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();
        if path_str.len() > self.config.max_path_length {
            return Err(anyhow!(
                "Path too long: {} characters exceeds limit of {}",
                path_str.len(),
                self.config.max_path_length
            ));
        }
        Ok(())
    }

    /// Validate text encoding
    fn validate_encoding(&self, input: &str) -> Result<()> {
        // Check if it's valid UTF-8 (Rust strings are always valid UTF-8)
        // But we can check for specific problematic characters

        if !self.config.allow_unicode && !input.is_ascii() {
            return Err(anyhow!("Content contains non-ASCII characters"));
        }

        // Check for null bytes
        if input.contains('\0') {
            return Err(anyhow!("Content contains null bytes"));
        }

        // Check for control characters (except common ones like \n, \r, \t)
        for ch in input.chars() {
            if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
                return Err(anyhow!(
                    "Content contains invalid control character: {:?}",
                    ch
                ));
            }
        }

        Ok(())
    }

    /// Validate file size
    pub fn validate_file_size(&self, size: usize) -> Result<()> {
        if size > self.config.max_file_size {
            return Err(anyhow!(
                "File size {} bytes exceeds limit of {} bytes",
                size,
                self.config.max_file_size
            ));
        }
        Ok(())
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_requirements() {
        let validator = InputValidator::new();

        // Valid requirements
        assert!(validator
            .validate_requirements("Build a task management system")
            .is_ok());

        // Empty requirements
        assert!(validator.validate_requirements("").is_err());

        // Too long
        let long_text = "a".repeat(2 * 1024 * 1024);
        assert!(validator.validate_requirements(&long_text).is_err());

        // Path traversal
        assert!(validator
            .validate_requirements("Requirements with ../ path")
            .is_err());
    }

    #[test]
    fn test_validate_project_name() {
        let validator = InputValidator::new();

        // Valid names
        assert!(validator.validate_project_name("my-project").is_ok());
        assert!(validator.validate_project_name("project_123").is_ok());

        // Invalid names
        assert!(validator.validate_project_name("").is_err());
        assert!(validator.validate_project_name("my project").is_err());
        assert!(validator.validate_project_name("my/project").is_err());
        assert!(validator.validate_project_name("con").is_err());
        assert!(validator.validate_project_name(".").is_err());
    }

    #[test]
    fn test_sanitize_filename() {
        let validator = InputValidator::new();

        assert_eq!(validator.sanitize_filename("my file.txt"), "myfile.txt");
        assert_eq!(
            validator.sanitize_filename("test@#$%file.md"),
            "testfile.md"
        );
    }

    #[test]
    fn test_validate_filename() {
        let validator = InputValidator::new();

        // Valid filenames
        assert!(validator.validate_filename("test.txt").is_ok());
        assert!(validator.validate_filename("my-file_123.md").is_ok());

        // Invalid filenames
        assert!(validator.validate_filename("").is_err());
        assert!(validator.validate_filename("my file.txt").is_err());
        assert!(validator.validate_filename("con.txt").is_err());
    }

    #[test]
    fn test_validate_encoding() {
        let validator = InputValidator::new();

        // Valid text
        assert!(validator.validate_requirements("Hello, World!").is_ok());
        assert!(validator.validate_requirements("Hello\nWorld").is_ok());

        // Unicode (should be allowed by default)
        assert!(validator.validate_requirements("Hello 世界").is_ok());

        // Null bytes
        assert!(validator.validate_requirements("Hello\0World").is_err());
    }

    #[test]
    fn test_validate_file_size() {
        let validator = InputValidator::new();

        assert!(validator.validate_file_size(1024).is_ok());
        assert!(validator.validate_file_size(100 * 1024 * 1024).is_err());
    }
}

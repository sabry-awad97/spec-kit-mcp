//! Utility Functions
//!
//! Common utilities used across the spec-kit MCP server.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Validate that the project has been initialized with spec-kit
pub fn validate_project_initialized() -> Result<(), String> {
    if !Path::new(".specify").exists() {
        Err("Error: .specify directory not found!\n\n\
             Please run speckit_init first to initialize the project structure.\n\n\
             Example: Use speckit_init with project_name=\"my-project\""
            .to_string())
    } else {
        Ok(())
    }
}

/// Validate that a file path is safe (within project directory)
pub fn validate_safe_path(path: &Path) -> Result<PathBuf> {
    // For relative paths that don't exist yet, just ensure they don't try to escape
    if path.is_relative() {
        // Check for directory traversal attempts
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            // Allow .. only if it doesn't escape the current directory
            let normalized = path
                .components()
                .fold(PathBuf::new(), |mut acc, component| {
                    match component {
                        std::path::Component::ParentDir => {
                            acc.pop();
                        }
                        std::path::Component::Normal(name) => {
                            acc.push(name);
                        }
                        _ => {}
                    }
                    acc
                });

            // If normalized path is empty or starts with .., it's trying to escape
            if normalized.components().next().is_none()
                || matches!(
                    normalized.components().next(),
                    Some(std::path::Component::ParentDir)
                )
            {
                anyhow::bail!(
                    "Security error: Path '{}' attempts to escape the project directory",
                    path.display()
                );
            }

            return Ok(normalized);
        }

        // Simple relative path, just return it
        return Ok(path.to_path_buf());
    }

    // For absolute paths, ensure they're within the current directory
    let canonical = if path.exists() {
        path.canonicalize().context("Failed to canonicalize path")?
    } else {
        // If path doesn't exist yet, canonicalize parent and append filename
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid path: no parent directory"))?;

        let parent_canonical = if parent.as_os_str().is_empty() {
            std::env::current_dir().context("Failed to get current directory")?
        } else {
            parent
                .canonicalize()
                .context("Failed to canonicalize parent directory")?
        };

        let filename = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid path: no filename"))?;

        parent_canonical.join(filename)
    };

    // Get current working directory
    let cwd = std::env::current_dir().context("Failed to get current directory")?;

    // Ensure the path is within the current directory
    if !canonical.starts_with(&cwd) {
        anyhow::bail!(
            "Security error: Path '{}' is outside the project directory",
            path.display()
        );
    }

    Ok(canonical)
}

/// Validate that a required file exists
pub fn validate_file_exists(path: &Path, file_type: &str) -> Result<(), String> {
    if !path.exists() {
        Err(format!(
            "Error: {} file not found at {}\n\n\
             Please create the {} first using the appropriate speckit tool.",
            file_type,
            path.display(),
            file_type
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_validate_project_initialized_missing() {
        let dir = tempdir().unwrap();
        let _guard = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = validate_project_initialized();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".specify directory not found"));

        std::env::set_current_dir(_guard).unwrap();
    }

    #[test]
    fn test_validate_project_initialized_exists() {
        let dir = tempdir().unwrap();
        let specify_dir = dir.path().join(".specify");
        fs::create_dir(&specify_dir).unwrap();

        let _guard = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = validate_project_initialized();
        assert!(result.is_ok());

        std::env::set_current_dir(_guard).unwrap();
    }

    #[test]
    fn test_validate_safe_path_within_project() {
        let dir = tempdir().unwrap();
        let _guard = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let safe_path = Path::new("test.txt");
        let result = validate_safe_path(safe_path);
        assert!(result.is_ok());

        std::env::set_current_dir(_guard).unwrap();
    }

    #[test]
    fn test_validate_safe_path_outside_project() {
        let dir = tempdir().unwrap();
        let _guard = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let unsafe_path = Path::new("../../../etc/passwd");
        let result = validate_safe_path(unsafe_path);
        assert!(result.is_err());

        std::env::set_current_dir(_guard).unwrap();
    }

    #[test]
    fn test_validate_file_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        let result = validate_file_exists(&file_path, "test file");
        assert!(result.is_ok());

        let missing_path = dir.path().join("missing.txt");
        let result = validate_file_exists(&missing_path, "test file");
        assert!(result.is_err());
    }
}

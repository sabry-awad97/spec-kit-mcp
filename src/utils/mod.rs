//! Utility Functions
//!
//! Common utilities used across the spec-kit MCP server.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Validate that the project has been initialized with spec-kit
/// If a path is provided, check for .specify in the parent directory of that path
pub fn validate_project_initialized_for_path(path: Option<&Path>) -> Result<(), String> {
    let check_dir = if let Some(p) = path {
        // Extract the directory from the path
        if let Some(parent) = p.parent() {
            if parent.as_os_str().is_empty() {
                Path::new(".")
            } else {
                parent
            }
        } else {
            Path::new(".")
        }
    } else {
        Path::new(".")
    };

    let specify_dir = check_dir.join(".specify");

    if !specify_dir.exists() {
        Err(format!(
            "Error: .specify directory not found in '{}'!\n\n\
             Please ensure you're working in an initialized spec-kit project.\n\n\
             If the project is in a subdirectory, make sure your output_path includes that directory.\n\
             Example: output_path=\"my-project/speckit.constitution\"",
            check_dir.display()
        ))
    } else {
        Ok(())
    }
}

/// Validate that the project has been initialized with spec-kit (checks current directory)
pub fn validate_project_initialized() -> Result<(), String> {
    validate_project_initialized_for_path(None)
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

        // Try to escape with enough .. to definitely go outside
        // Use an absolute path to a system directory to ensure it's outside
        let system_path = if cfg!(windows) {
            Path::new("C:\\Windows\\System32\\config")
        } else {
            Path::new("/etc/passwd")
        };

        let result = validate_safe_path(system_path);
        assert!(
            result.is_err(),
            "Should reject absolute path outside project"
        );

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

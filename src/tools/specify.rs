//! Spec-Kit Specify Tool
//!
//! Defines requirements and user stories.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::speckit::SpecKitCli;
use crate::tools::Tool;

/// Parameters for the speckit_specify tool
#[derive(Debug, Deserialize, Serialize)]
pub struct SpecifyParams {
    /// Requirements to specify
    requirements: String,

    /// User stories (optional)
    #[serde(default)]
    user_stories: Option<String>,

    /// Output path for specification file
    #[serde(default = "default_specify_path")]
    output_path: PathBuf,

    /// Output format
    #[serde(default = "default_format")]
    format: String,
}

fn default_specify_path() -> PathBuf {
    PathBuf::from("./speckit.specify")
}

fn default_format() -> String {
    "markdown".to_string()
}

/// Tool for creating specifications
pub struct SpecifyTool {
    #[allow(dead_code)]
    cli: SpecKitCli,
}

impl SpecifyTool {
    /// Create a new specify tool
    pub fn new(cli: SpecKitCli) -> Self {
        Self { cli }
    }
}

#[async_trait]
impl Tool for SpecifyTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "speckit_specify".to_string(),
            description: "Define what you want to build - requirements, user stories, and acceptance criteria".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "requirements": {
                        "type": "string",
                        "description": "The requirements to specify. Can include features, constraints, user needs, etc."
                    },
                    "user_stories": {
                        "type": "string",
                        "description": "Optional user stories in 'As a... I want... So that...' format"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Path where the specification file will be written",
                        "default": "./speckit.specify"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["markdown", "yaml", "json"],
                        "default": "markdown",
                        "description": "Output format for the specification"
                    }
                },
                "required": ["requirements"]
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: SpecifyParams =
            serde_json::from_value(params).context("Failed to parse specify parameters")?;

        tracing::info!(
            output_path = %params.output_path.display(),
            format = %params.format,
            "Creating specification"
        );

        // Check if .specify directory exists
        let specify_dir = std::path::Path::new(".specify");
        if !specify_dir.exists() {
            return Ok(ToolResult {
                content: vec![ContentBlock::text(
                    "Error: .specify directory not found!\n\n\
                    Please run speckit_init first to initialize the project structure.\n\n\
                    Example: Use speckit_init with project_name=\"my-project\""
                        .to_string(),
                )],
                is_error: Some(true),
            });
        }

        // Format the specification content
        let mut content = format!(
            "# Specification\n\n\
            ## Requirements\n\n{}\n",
            params.requirements
        );

        if let Some(stories) = params.user_stories {
            content.push_str(&format!("\n## User Stories\n\n{}\n", stories));
        }

        // Ensure parent directory exists
        if let Some(parent) = params.output_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create parent directory")?;
        }

        // Write specification file
        tokio::fs::write(&params.output_path, content)
            .await
            .context("Failed to write specification file")?;

        let message = format!(
            "Specification created successfully at {}\n\n\
            The specification defines:\n\
            - What needs to be built (requirements)\n\
            - Who it's for and why (user stories)\n\
            - Success criteria (acceptance criteria)\n\n\
            Next step: Use speckit_plan tool to create a technical plan",
            params.output_path.display()
        );

        Ok(ToolResult {
            content: vec![ContentBlock::text(message)],
            is_error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_specify_tool_definition() {
        let cli = SpecKitCli::new();
        let tool = SpecifyTool::new(cli);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_specify");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_specify_tool_execute() {
        let cli = SpecKitCli::new_test_mode();
        let tool = SpecifyTool::new(cli);

        let dir = tempdir().unwrap();

        // Create .specify directory to simulate initialized project
        let specify_dir = dir.path().join(".specify");
        tokio::fs::create_dir(&specify_dir).await.unwrap();

        let output_path = dir.path().join("specification.md");

        let params = json!({
            "requirements": "User authentication system with OAuth2 support",
            "user_stories": "As a user, I want to login with Google, so that I don't need another password",
            "output_path": output_path.to_str().unwrap()
        });

        // For testing, temporarily create .specify in current directory
        let current_specify = std::path::Path::new(".specify");
        let cleanup_needed = !current_specify.exists();
        if cleanup_needed {
            tokio::fs::create_dir(current_specify).await.unwrap();
        }

        let result = tool.execute(params).await.unwrap();

        // Cleanup
        if cleanup_needed {
            tokio::fs::remove_dir(current_specify).await.ok();
        }

        assert!(result.is_error.is_none() || !result.is_error.unwrap());
        assert!(output_path.exists());
    }
}

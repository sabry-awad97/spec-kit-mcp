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
use crate::utils::{validate_project_initialized, validate_safe_path};

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

        // Validate project is initialized
        if let Err(msg) = validate_project_initialized() {
            return Ok(ToolResult {
                content: vec![ContentBlock::text(msg)],
                is_error: Some(true),
            });
        }

        // Validate output path is safe
        let safe_path = match validate_safe_path(&params.output_path) {
            Ok(path) => path,
            Err(e) => {
                return Ok(ToolResult {
                    content: vec![ContentBlock::text(format!(
                        "Invalid output path: {}\n\n\
                        Please provide a path within the project directory.",
                        e
                    ))],
                    is_error: Some(true),
                });
            }
        };

        // Get the specification template
        let template = crate::templates::SPEC_TEMPLATE;

        // Format the specification content using the template
        let mut content = template
            .replace("[FEATURE NAME]", "Feature")
            .replace("$ARGUMENTS", &params.requirements);

        // Add user stories if provided
        if let Some(stories) = params.user_stories {
            content = content.replace(
                "### User Story 1 - [Brief Title] (Priority: P1)",
                &format!("### User Stories\n\n{}", stories),
            );
        }

        // Ensure parent directory exists
        if let Some(parent) = safe_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }

        // Write specification file
        tokio::fs::write(&safe_path, content)
            .await
            .with_context(|| {
                format!(
                    "Failed to write specification file to: {}",
                    safe_path.display()
                )
            })?;

        let message = format!(
            "Specification created successfully at {}\n\n\
            The specification defines:\n\
            - What needs to be built (requirements)\n\
            - Who it's for and why (user stories)\n\
            - Success criteria (acceptance criteria)\n\n\
            Next step: Use speckit_plan tool to create a technical plan\n\n\
            ---\n\n\
            ## How to use this tool\n\n\
            {}",
            safe_path.display(),
            crate::templates::SPECIFY_COMMAND
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
            "output_path": "specification.md"  // Use relative path
        });

        // Change to temp directory for test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = tool.execute(params).await.unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Check result - should succeed now that we're in the right directory
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
        assert!(output_path.exists());
    }
}

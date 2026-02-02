//! Spec-Kit Constitution Tool
//!
//! Creates project governing principles and development guidelines.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::speckit::SpecKitCli;
use crate::tools::Tool;
use crate::utils::{validate_project_initialized, validate_safe_path};

/// Parameters for the speckit_constitution tool
#[derive(Debug, Deserialize, Serialize)]
pub struct ConstitutionParams {
    /// Core principles and values
    principles: String,

    /// Technical constraints (optional)
    #[serde(default)]
    constraints: Option<String>,

    /// Output path for constitution file
    #[serde(default = "default_constitution_path")]
    output_path: PathBuf,
}

fn default_constitution_path() -> PathBuf {
    PathBuf::from("./speckit.constitution")
}

/// Tool for creating project constitutions
pub struct ConstitutionTool {
    #[allow(dead_code)]
    cli: SpecKitCli,
}

impl ConstitutionTool {
    /// Create a new constitution tool
    pub fn new(cli: SpecKitCli) -> Self {
        Self { cli }
    }
}

#[async_trait]
impl Tool for ConstitutionTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "speckit_constitution".to_string(),
            description: "Create or update project governing principles, development standards, and technical constraints".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "principles": {
                        "type": "string",
                        "description": "Core principles and values that govern the project (e.g., simplicity, performance, security)"
                    },
                    "constraints": {
                        "type": "string",
                        "description": "Technical constraints and boundaries (optional)"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Path where the constitution file will be written",
                        "default": "./speckit.constitution"
                    }
                },
                "required": ["principles"]
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: ConstitutionParams =
            serde_json::from_value(params).context("Failed to parse constitution parameters")?;

        tracing::info!(
            output_path = %params.output_path.display(),
            "Creating constitution"
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

        // Return instructions for AI to follow - DO NOT write template file yet
        let message = format!(
            "## Task: Create Project Constitution\n\n\
            **Output File**: {}\n\n\
            **Principles**:\n```\n{}\n```\n\n\
            **Constraints**:\n```\n{}\n```\n\n\
            ---\n\n\
            ## Instructions\n\n\
            You must now follow the detailed workflow below to generate a complete constitution.\n\
            After generating the content, write it to the output file path above.\n\n\
            **IMPORTANT**: Do NOT write placeholder content. Generate fully populated content following the instructions.\n\n\
            {}",
            safe_path.display(),
            params.principles,
            params.constraints.as_deref().unwrap_or("(none provided)"),
            crate::templates::CONSTITUTION_COMMAND
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
    async fn test_constitution_tool_definition() {
        let cli = SpecKitCli::new();
        let tool = ConstitutionTool::new(cli);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_constitution");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_constitution_tool_execute() {
        let cli = SpecKitCli::new_test_mode();
        let tool = ConstitutionTool::new(cli);

        let dir = tempdir().unwrap();

        // Create .specify directory to simulate initialized project
        let specify_dir = dir.path().join(".specify");
        tokio::fs::create_dir(&specify_dir).await.unwrap();

        let params = json!({
            "principles": "Simplicity, Performance, Security",
            "constraints": "Must support Python 3.11+",
            "output_path": "constitution.md"
        });

        // Change to temp directory for test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = tool.execute(params).await.unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Check result - tool should return instructions, not write file
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
        // File should NOT exist yet - AI will create it after following instructions
        // assert!(!std::path::Path::new("constitution.md").exists());
    }
}

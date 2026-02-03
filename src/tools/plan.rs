//! Spec-Kit Plan Tool
//!
//! Creates technical implementation plans.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::tools::Tool;
use crate::utils::{
    validate_file_exists, validate_project_initialized_for_path, validate_safe_path,
};
use crate::validation::InputValidator;

/// Parameters for the speckit_plan tool
#[derive(Debug, Deserialize, Serialize)]
pub struct PlanParams {
    /// Path to specification file
    spec_file: PathBuf,

    /// Tech stack to use
    #[serde(default)]
    tech_stack: Option<String>,

    /// Output path for plan file
    #[serde(default = "default_plan_path")]
    output_path: PathBuf,
}

fn default_plan_path() -> PathBuf {
    PathBuf::from("./speckit.plan")
}

/// Tool for creating technical plans
pub struct PlanTool {
    #[allow(dead_code)] // Will be used for future validation
    validator: InputValidator,
}

impl PlanTool {
    /// Create a new plan tool
    pub fn new(validator: InputValidator) -> Self {
        Self { validator }
    }
}

#[async_trait]
impl Tool for PlanTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "speckit_plan".to_string(),
            description: "Create a technical implementation plan based on the specification, including architecture, tech stack, and approach".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "spec_file": {
                        "type": "string",
                        "description": "Path to the specification file (speckit.specify)"
                    },
                    "tech_stack": {
                        "type": "string",
                        "description": "Technology stack to use (e.g., 'Rust + Tokio', 'Python + FastAPI')"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Path where the plan file will be written",
                        "default": "./speckit.plan"
                    }
                },
                "required": ["spec_file"]
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: PlanParams =
            serde_json::from_value(params).context("Failed to parse plan parameters")?;

        tracing::info!(
            spec_file = %params.spec_file.display(),
            output_path = %params.output_path.display(),
            "Creating technical plan"
        );

        // Validate project is initialized (check in the output path's directory)
        if let Err(msg) = validate_project_initialized_for_path(Some(&params.output_path)) {
            return Ok(ToolResult {
                content: vec![ContentBlock::text(msg)],
                is_error: Some(true),
            });
        }

        // Validate spec file exists
        if let Err(msg) = validate_file_exists(&params.spec_file, "Specification") {
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

        // Read the specification
        let spec_content = tokio::fs::read_to_string(&params.spec_file)
            .await
            .with_context(|| {
                format!(
                    "Failed to read specification file: {}",
                    params.spec_file.display()
                )
            })?;

        // Return instructions for AI to follow - DO NOT write template file yet
        let message = format!(
            "## Task: Create Technical Implementation Plan\n\n\
            **Output File**: {}\n\n\
            **Specification File**: {}\n\n\
            **Tech Stack**:\n```\n{}\n```\n\n\
            **Specification Content**:\n```\n{}\n```\n\n\
            ---\n\n\
            ## Instructions\n\n\
            You must now follow the detailed workflow below to generate a complete technical plan.\n\
            After generating the content, write it to the output file path above.\n\n\
            **IMPORTANT**: Do NOT write placeholder content. Generate fully populated content following the instructions.\n\n\
            {}",
            safe_path.display(),
            params.spec_file.display(),
            params.tech_stack.as_deref().unwrap_or("(not specified - you should determine appropriate stack)"),
            spec_content,
            crate::templates::PLAN_COMMAND
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
    use tokio::fs;

    #[tokio::test]
    async fn test_plan_tool_definition() {
        let validator = InputValidator::new();
        let tool = PlanTool::new(validator);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_plan");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_plan_tool_execute() {
        let validator = InputValidator::new();
        let tool = PlanTool::new(validator);

        let dir = tempdir().unwrap();

        // Create .specify directory to simulate initialized project
        let specify_dir = dir.path().join(".specify");
        fs::create_dir(&specify_dir).await.unwrap();

        // Create dummy spec file using absolute path
        let spec_file = dir.path().join("spec.md");
        fs::write(&spec_file, "Test specification").await.unwrap();

        // Change to temp directory for validation
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let params = json!({
            "spec_file": "spec.md",
            "tech_stack": "Rust + Tokio",
            "output_path": "plan.md"
        });

        let result = tool.execute(params).await;

        // Restore original directory before asserting
        std::env::set_current_dir(original_dir).unwrap();

        // Check result - tool should return instructions, not write file
        let result = result.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
    }
}

//! Spec-Kit Plan Tool
//!
//! Creates technical implementation plans.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::speckit::SpecKitCli;
use crate::tools::Tool;

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
    #[allow(dead_code)]
    cli: SpecKitCli,
}

impl PlanTool {
    /// Create a new plan tool
    pub fn new(cli: SpecKitCli) -> Self {
        Self { cli }
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

        // Check if spec file exists
        if !params.spec_file.exists() {
            return Ok(ToolResult {
                content: vec![ContentBlock::text(format!(
                    "Error: Specification file not found at {}\n\n\
                    Please create a specification first using speckit_specify tool.",
                    params.spec_file.display()
                ))],
                is_error: Some(true),
            });
        }

        // Read the specification
        let spec_content = tokio::fs::read_to_string(&params.spec_file)
            .await
            .context("Failed to read specification file")?;

        // Create a basic plan template
        let mut content = format!(
            "# Technical Implementation Plan\n\n\
            ## Based on Specification\n\n\
            Source: {}\n\n\
            ## Architecture\n\n\
            [AI should fill in architecture details based on the specification]\n\n\
            ## Technology Stack\n\n",
            params.spec_file.display()
        );

        if let Some(tech_stack) = params.tech_stack {
            content.push_str(&format!("{}\n\n", tech_stack));
        } else {
            content.push_str("[AI should determine appropriate tech stack]\n\n");
        }

        content.push_str(
            "## Implementation Approach\n\n\
            [AI should detail the implementation approach]\n\n\
            ## Module Breakdown\n\n\
            [AI should break down into modules/components]\n\n\
            ## Specification Reference\n\n```\n",
        );
        content.push_str(&spec_content);
        content.push_str("\n```\n");

        // Ensure parent directory exists
        if let Some(parent) = params.output_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create parent directory")?;
        }

        // Write plan file
        tokio::fs::write(&params.output_path, content)
            .await
            .context("Failed to write plan file")?;

        let message = format!(
            "Technical plan created successfully at {}\n\n\
            The plan includes:\n\
            - Architecture and system design\n\
            - Technology stack and frameworks\n\
            - Implementation approach\n\
            - Module breakdown\n\n\
            Next step: Use speckit_tasks tool to generate actionable tasks",
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
    use tokio::fs;

    #[tokio::test]
    async fn test_plan_tool_definition() {
        let cli = SpecKitCli::new();
        let tool = PlanTool::new(cli);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_plan");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_plan_tool_execute() {
        let cli = SpecKitCli::new_test_mode();
        let tool = PlanTool::new(cli);

        let dir = tempdir().unwrap();

        // Create .specify directory to simulate initialized project
        let specify_dir = dir.path().join(".specify");
        fs::create_dir(&specify_dir).await.unwrap();

        let spec_file = dir.path().join("spec.md");
        let output_path = dir.path().join("plan.md");

        // Create dummy spec file
        fs::write(&spec_file, "Test specification").await.unwrap();

        let params = json!({
            "spec_file": spec_file.to_str().unwrap(),
            "tech_stack": "Rust + Tokio",
            "output_path": output_path.to_str().unwrap()
        });

        // For testing, temporarily create .specify in current directory
        let current_specify = std::path::Path::new(".specify");
        let cleanup_needed = !current_specify.exists();
        if cleanup_needed {
            fs::create_dir(current_specify).await.unwrap();
        }

        let result = tool.execute(params).await.unwrap();

        // Cleanup
        if cleanup_needed {
            fs::remove_dir(current_specify).await.ok();
        }

        assert!(result.is_error.is_none() || !result.is_error.unwrap());
    }
}

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
use crate::utils::{validate_file_exists, validate_project_initialized, validate_safe_path};

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

        // Validate project is initialized
        if let Err(msg) = validate_project_initialized() {
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

        // Get the plan template
        let template = crate::templates::PLAN_TEMPLATE;

        // Create a basic plan using the template
        let mut content = template.replace("[FEATURE]", "Feature").replace(
            "[DATE]",
            &chrono::Local::now().format("%Y-%m-%d").to_string(),
        );

        // Add tech stack if provided
        if let Some(tech_stack) = params.tech_stack {
            content = content.replace(
                "**Language/Version**: [e.g., Typescript, Python 3.11, Swift 5.9, Rust 1.75 or NEEDS CLARIFICATION]",
                &format!("**Language/Version**: {}", tech_stack)
            );
        }

        // Add reference to the specification
        content.push_str(&format!(
            "\n\n## Specification Reference\n\n```\n{}\n```\n",
            spec_content
        ));

        // Ensure parent directory exists
        if let Some(parent) = safe_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }

        // Write plan file
        tokio::fs::write(&safe_path, content)
            .await
            .with_context(|| format!("Failed to write plan file to: {}", safe_path.display()))?;

        let message = format!(
            "Technical plan created successfully at {}\n\n\
            The plan includes:\n\
            - Architecture and system design\n\
            - Technology stack and frameworks\n\
            - Implementation approach\n\
            - Module breakdown\n\n\
            Next step: Use speckit_tasks tool to generate actionable tasks\n\n\
            ---\n\n\
            ## How to use this tool\n\n\
            {}",
            safe_path.display(),
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
        let _output_path = dir.path().join("plan.md");

        // Create dummy spec file
        fs::write(&spec_file, "Test specification").await.unwrap();

        let params = json!({
            "spec_file": "spec.md",  // Use relative path
            "tech_stack": "Rust + Tokio",
            "output_path": "plan.md"  // Use relative path
        });

        // Change to temp directory for test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = tool.execute(params).await.unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Check result - should succeed now that we're in the right directory
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
    }
}

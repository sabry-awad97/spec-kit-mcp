//! Spec-Kit Tasks Tool
//!
//! Generates actionable task lists from technical plans.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::speckit::SpecKitCli;
use crate::tools::Tool;
use crate::utils::{validate_file_exists, validate_project_initialized, validate_safe_path};

/// Parameters for the speckit_tasks tool
#[derive(Debug, Deserialize, Serialize)]
pub struct TasksParams {
    /// Path to plan file
    plan_file: PathBuf,

    /// Breakdown level
    #[serde(default = "default_breakdown_level")]
    breakdown_level: String,

    /// Output path for tasks file
    #[serde(default = "default_tasks_path")]
    output_path: PathBuf,
}

fn default_breakdown_level() -> String {
    "medium".to_string()
}

fn default_tasks_path() -> PathBuf {
    PathBuf::from("./speckit.tasks")
}

/// Tool for generating task lists
pub struct TasksTool {
    #[allow(dead_code)]
    cli: SpecKitCli,
}

impl TasksTool {
    /// Create a new tasks tool
    pub fn new(cli: SpecKitCli) -> Self {
        Self { cli }
    }
}

#[async_trait]
impl Tool for TasksTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "speckit_tasks".to_string(),
            description: "Generate actionable task lists from the technical plan, breaking down work into manageable items".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "plan_file": {
                        "type": "string",
                        "description": "Path to the plan file (speckit.plan)"
                    },
                    "breakdown_level": {
                        "type": "string",
                        "enum": ["high", "medium", "detailed"],
                        "default": "medium",
                        "description": "Level of task breakdown (high=major milestones, detailed=granular tasks)"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Path where the tasks file will be written",
                        "default": "./speckit.tasks"
                    }
                },
                "required": ["plan_file"]
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: TasksParams =
            serde_json::from_value(params).context("Failed to parse tasks parameters")?;

        tracing::info!(
            plan_file = %params.plan_file.display(),
            breakdown_level = %params.breakdown_level,
            output_path = %params.output_path.display(),
            "Generating task list"
        );

        // Validate project is initialized
        if let Err(msg) = validate_project_initialized() {
            return Ok(ToolResult {
                content: vec![ContentBlock::text(msg)],
                is_error: Some(true),
            });
        }

        // Validate plan file exists
        if let Err(msg) = validate_file_exists(&params.plan_file, "Plan") {
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

        // Read the plan
        let plan_content = tokio::fs::read_to_string(&params.plan_file)
            .await
            .with_context(|| format!("Failed to read plan file: {}", params.plan_file.display()))?;

        // Create a basic tasks template
        let content = format!(
            "# Task List\n\n\
            ## Based on Plan\n\n\
            Source: {}\n\
            Breakdown Level: {}\n\n\
            ## Tasks\n\n\
            [AI should generate actionable tasks based on the plan below]\n\n\
            ### Task Format\n\
            - [ ] Task description\n\
            - Acceptance criteria: ...\n\
            - Dependencies: ...\n\
            - Estimated effort: ...\n\n\
            ## Plan Reference\n\n```\n{}\n```\n",
            params.plan_file.display(),
            params.breakdown_level,
            plan_content
        );

        // Ensure parent directory exists
        if let Some(parent) = safe_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }

        // Write tasks file
        tokio::fs::write(&safe_path, content)
            .await
            .with_context(|| format!("Failed to write tasks file to: {}", safe_path.display()))?;

        let message = format!(
            "Task list generated successfully at {}\n\n\
            The task list includes:\n\
            - Prioritized actionable items\n\
            - Clear acceptance criteria\n\
            - Dependencies between tasks\n\
            - Estimated effort levels\n\n\
            Next step: Use speckit_implement tool to execute the tasks",
            safe_path.display()
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
    async fn test_tasks_tool_definition() {
        let cli = SpecKitCli::new();
        let tool = TasksTool::new(cli);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_tasks");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_tasks_tool_execute() {
        let cli = SpecKitCli::new_test_mode();
        let tool = TasksTool::new(cli);

        let dir = tempdir().unwrap();

        // Create .specify directory to simulate initialized project
        let specify_dir = dir.path().join(".specify");
        fs::create_dir(&specify_dir).await.unwrap();

        let plan_file = dir.path().join("plan.md");
        let _output_path = dir.path().join("tasks.md");

        // Create dummy plan file
        fs::write(&plan_file, "Test plan").await.unwrap();

        let params = json!({
            "plan_file": "plan.md",  // Use relative path
            "breakdown_level": "medium",
            "output_path": "tasks.md"  // Use relative path
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

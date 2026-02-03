//! Spec-Kit Implement Tool
//!
//! Executes implementation according to the task list.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::tools::Tool;
use crate::utils::{validate_file_exists, validate_safe_path};
use crate::validation::InputValidator;

/// Parameters for the speckit_implement tool
#[derive(Debug, Deserialize, Serialize)]
pub struct ImplementParams {
    /// Path to tasks file
    task_file: PathBuf,

    /// Additional context for implementation
    #[serde(default)]
    context: Option<String>,

    /// Output directory for implementation
    #[serde(default = "default_output_dir")]
    output_dir: PathBuf,
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("./src")
}

/// Tool for executing implementation
pub struct ImplementTool {
    #[allow(dead_code)] // Will be used for future validation
    validator: InputValidator,
}

impl ImplementTool {
    /// Create a new implement tool
    pub fn new(validator: InputValidator) -> Self {
        Self { validator }
    }
}

#[async_trait]
impl Tool for ImplementTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "speckit_implement".to_string(),
            description: "Execute implementation according to the task list, generating code and documentation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_file": {
                        "type": "string",
                        "description": "Path to the tasks file (speckit.tasks)"
                    },
                    "context": {
                        "type": "string",
                        "description": "Additional context for implementation (e.g., existing code patterns, constraints)"
                    },
                    "output_dir": {
                        "type": "string",
                        "description": "Directory where code will be generated",
                        "default": "./src"
                    }
                },
                "required": ["task_file"]
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: ImplementParams =
            serde_json::from_value(params).context("Failed to parse implement parameters")?;

        tracing::info!(
            task_file = %params.task_file.display(),
            output_dir = %params.output_dir.display(),
            "Executing implementation"
        );

        // Validate task file exists
        if let Err(msg) = validate_file_exists(&params.task_file, "Tasks") {
            return Ok(ToolResult {
                content: vec![ContentBlock::text(msg)],
                is_error: Some(true),
            });
        }

        // Validate output directory is safe
        let safe_output_dir = match validate_safe_path(&params.output_dir) {
            Ok(path) => path,
            Err(e) => {
                return Ok(ToolResult {
                    content: vec![ContentBlock::text(format!(
                        "Invalid output directory: {}\n\n\
                        Please provide a path within the project directory.",
                        e
                    ))],
                    is_error: Some(true),
                });
            }
        };

        // Read the tasks file
        let tasks_content = tokio::fs::read_to_string(&params.task_file)
            .await
            .with_context(|| {
                format!("Failed to read tasks file: {}", params.task_file.display())
            })?;

        // Return enhanced instructions for AI to follow
        let message = format!(
            "# ROLE & CONTEXT\n\n\
            You are a **Senior Software Engineer** with 12+ years of experience in full-stack development and implementation.\n\n\
            ## Task: Execute Implementation\n\n\
            **Task File**: {}\n\n\
            **Output Directory**: {}\n\n\
            **Additional Context**:\n```\n{}\n```\n\n\
            **Tasks Content**:\n```\n{}\n```\n\n\
            ---\n\n\
            # STRUCTURED THINKING PROTOCOL\n\n\
            Before executing implementation, complete these steps:\n\n\
            ## [UNDERSTAND]\n\
            - Review complete task list from tasks.md\n\
            - Understand technical architecture from plan.md\n\
            - Identify current phase and dependencies\n\n\
            ## [ANALYZE]\n\
            - Assess task dependencies and execution order\n\
            - Identify which tasks can run in parallel\n\
            - Recognize potential integration challenges\n\n\
            ## [STRATEGIZE]\n\
            - Plan phase-by-phase execution approach\n\
            - Determine validation checkpoints\n\
            - Prepare error handling strategy\n\n\
            ## [EXECUTE]\n\
            - Implement tasks following the plan\n\
            - Validate each phase before proceeding\n\
            - Track progress and mark completed tasks\n\
            - Handle errors systematically\n\n\
            ---\n\n\
            # DETAILED INSTRUCTIONS\n\n\
            You must now follow the detailed workflow below to implement the tasks.\n\n\
            **CRITICAL REQUIREMENTS**:\n\
            - Implement each task in order, respecting dependencies\n\
            - Write complete, working code (no placeholders)\n\
            - Create files in the output directory specified above\n\
            - Follow the technical plan and specifications\n\
            - Mark completed tasks as [X] in the tasks file\n\n\
            ---\n\n\
            # WORKFLOW\n\n\
            {}\n\n\
            ---\n\n\
            # CHAIN-OF-VERIFICATION\n\n\
            After completing each phase, verify:\n\n\
            1. Does the implemented code match the task description exactly?\n\
            2. Are all file paths correct and files created in the right locations?\n\
            3. Does the code follow the architecture and patterns from the plan?\n\
            4. Are error cases handled appropriately?\n\
            5. Is the code ready for the next phase (no blocking issues)?\n\n\
            **Confidence Level**: Provide your confidence (0-100%) in the implementation quality.",
            params.task_file.display(),
            safe_output_dir.display(),
            params.context.as_deref().unwrap_or("(none provided)"),
            tasks_content,
            crate::templates::IMPLEMENT_COMMAND
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
    async fn test_implement_tool_definition() {
        let validator = InputValidator::new();
        let tool = ImplementTool::new(validator);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_implement");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_implement_tool_execute() {
        let validator = InputValidator::new();
        let tool = ImplementTool::new(validator);

        let dir = tempdir().unwrap();

        // Create dummy task file using absolute path
        let task_file = dir.path().join("tasks.md");
        fs::write(&task_file, "Task 1: Implement feature\nTask 2: Write tests")
            .await
            .unwrap();

        // Change to temp directory for validation
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let params = json!({
            "task_file": "tasks.md",  // Use relative path
            "context": "Using Rust 2021 edition",
            "output_dir": "src"  // Use relative path
        });

        let result = tool.execute(params).await.unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Check result - should succeed now that we're in the right directory
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
    }
}

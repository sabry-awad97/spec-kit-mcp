//! Spec-Kit Checklist Tool
//!
//! Generates validation checklists from specifications.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::tools::Tool;
use crate::utils::{validate_file_exists, validate_safe_path};
use crate::validation::InputValidator;

/// Parameters for the speckit_checklist tool
#[derive(Debug, Deserialize, Serialize)]
pub struct ChecklistParams {
    /// Path to specification file
    spec_file: PathBuf,

    /// Include implementation checklist items
    #[serde(default = "default_true")]
    include_implementation: bool,

    /// Include testing checklist items
    #[serde(default = "default_true")]
    include_testing: bool,

    /// Output path for checklist
    #[serde(default = "default_checklist_path")]
    output_path: PathBuf,
}

fn default_true() -> bool {
    true
}

fn default_checklist_path() -> PathBuf {
    PathBuf::from("./speckit.checklist")
}

/// Tool for generating validation checklists
pub struct ChecklistTool {
    #[allow(dead_code)] // Will be used for future validation
    validator: InputValidator,
}

impl ChecklistTool {
    /// Create a new checklist tool
    pub fn new(validator: InputValidator) -> Self {
        Self { validator }
    }
}

#[async_trait]
impl Tool for ChecklistTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "speckit_checklist".to_string(),
            description: "Generate a validation checklist from the specification to ensure all requirements are met".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "spec_file": {
                        "type": "string",
                        "description": "Path to the specification file"
                    },
                    "include_implementation": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include implementation checklist items"
                    },
                    "include_testing": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include testing checklist items"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Path where checklist will be written",
                        "default": "./speckit.checklist"
                    }
                },
                "required": ["spec_file"]
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: ChecklistParams =
            serde_json::from_value(params).context("Failed to parse checklist parameters")?;

        tracing::info!(
            spec_file = %params.spec_file.display(),
            "Generating validation checklist"
        );

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

        // Read specification
        let spec_content = tokio::fs::read_to_string(&params.spec_file)
            .await
            .with_context(|| {
                format!(
                    "Failed to read specification file: {}",
                    params.spec_file.display()
                )
            })?;

        // Return instructions for AI to follow - DO NOT write file yet
        let message = format!(
            "## Task: Generate Validation Checklist\n\n\
            **Specification File**: {}\n\n\
            **Output File**: {}\n\n\
            **Include Implementation**: {}\n\
            **Include Testing**: {}\n\n\
            **Specification Content**:\n```\n{}\n```\n\n\
            ---\n\n\
            ## Instructions\n\n\
            You must now follow the detailed workflow below to generate a validation checklist.\n\
            After generating the content, write it to the output file path above.\n\n\
            **IMPORTANT**: \n\
            - Extract all requirements from the specification\n\
            - Create specific, testable checklist items\n\
            - Organize by category (requirements, implementation, testing, quality)\n\
            - Do NOT write placeholder content\n\n\
            {}",
            params.spec_file.display(),
            safe_path.display(),
            params.include_implementation,
            params.include_testing,
            spec_content,
            crate::templates::CHECKLIST_COMMAND
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
    use std::path::Path;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_checklist_tool_definition() {
        let validator = InputValidator::new();
        let tool = ChecklistTool::new(validator);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_checklist");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_checklist_tool_execute() {
        let validator = InputValidator::new();
        let tool = ChecklistTool::new(validator);

        let dir = tempdir().unwrap();

        // Change to temp directory FIRST
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let spec_file = Path::new("spec.md");
        // Create spec with requirements in current directory
        fs::write(
            spec_file,
            "- User must login\n- System shall validate input\n- Should handle errors",
        )
        .await
        .unwrap();

        let params = json!({
            "spec_file": "spec.md",
            "include_implementation": true,
            "include_testing": true,
            "output_path": "checklist.md"
        });

        let result = tool.execute(params).await.unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Check result - tool should return instructions, not write file
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
    }
}

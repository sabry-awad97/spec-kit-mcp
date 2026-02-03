//! Spec-Kit Constitution Tool
//!
//! Creates project governing principles and development guidelines.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::tools::Tool;
use crate::utils::{validate_project_initialized_for_path, validate_safe_path};
use crate::validation::InputValidator;

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
    validator: InputValidator,
}

impl ConstitutionTool {
    /// Create a new constitution tool
    pub fn new(validator: InputValidator) -> Self {
        Self { validator }
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

        // Validate principles
        if let Err(e) = self.validator.validate_principles(&params.principles) {
            return Ok(ToolResult {
                content: vec![ContentBlock::text(format!(
                    "Invalid principles: {}\n\nSuggestion: {}",
                    e, "Ensure principles are not empty and within size limits"
                ))],
                is_error: Some(true),
            });
        }

        // Validate constraints if provided
        if let Some(ref constraints) = params.constraints {
            if let Err(e) = self.validator.validate_user_stories(constraints) {
                return Ok(ToolResult {
                    content: vec![ContentBlock::text(format!(
                        "Invalid constraints: {}\n\nSuggestion: {}",
                        e, "Ensure constraints are within size limits"
                    ))],
                    is_error: Some(true),
                });
            }
        }

        // Validate project is initialized (check in the output path's directory)
        if let Err(msg) = validate_project_initialized_for_path(Some(&params.output_path)) {
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

        // Return enhanced instructions for AI to follow - DO NOT write template file yet
        let message = format!(
            "# ROLE & CONTEXT\n\n\
            You are a **Senior Engineering Manager** with 15+ years of experience in establishing engineering standards and governance.\n\n\
            ## Task: Create Project Constitution\n\n\
            **Output File**: {}\n\n\
            **Principles**:\n```\n{}\n```\n\n\
            **Constraints**:\n```\n{}\n```\n\n\
            ---\n\n\
            # STRUCTURED THINKING PROTOCOL\n\n\
            Before generating the constitution, complete these steps:\n\n\
            ## [UNDERSTAND]\n\
            - Review current constitution (if exists)\n\
            - Identify principles being added, modified, or removed\n\
            - Extract governance requirements\n\n\
            ## [ANALYZE]\n\
            - Assess impact of changes on existing projects\n\
            - Identify dependencies on other templates\n\
            - Recognize versioning implications\n\n\
            ## [STRATEGIZE]\n\
            - Plan version bump based on change type\n\
            - Determine which templates need updates\n\
            - Prepare consistency propagation checklist\n\n\
            ## [EXECUTE]\n\
            - Update constitution with concrete values\n\
            - Propagate changes to dependent artifacts\n\
            - Generate sync impact report\n\
            - Validate consistency\n\n\
            ---\n\n\
            # DETAILED INSTRUCTIONS\n\n\
            You must now follow the detailed workflow below to generate a complete constitution.\n\
            After generating the content, write it to the output file path above.\n\n\
            **CRITICAL REQUIREMENTS**:\n\
            - All principles MUST be clear, testable, and enforceable\n\
            - Version bump MUST follow semantic versioning\n\
            - All dependent templates MUST be identified and updated\n\
            - No contradictions between principles\n\n\
            ---\n\n\
            # WORKFLOW\n\n\
            {}\n\n\
            ---\n\n\
            # CHAIN-OF-VERIFICATION\n\n\
            After generating the constitution, verify:\n\n\
            1. Are all principles clear, testable, and enforceable?\n\
            2. Is the version bump appropriate for the type of changes made?\n\
            3. Have all dependent templates been identified and updated?\n\
            4. Are there any contradictions between principles?\n\
            5. Can developers easily understand what's required vs. recommended?\n\n\
            **Confidence Level**: Provide your confidence (0-100%) in the constitution quality.\n\n\
            **Key Assumptions**: List assumptions about project context.",
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
        let validator = InputValidator::new();
        let tool = ConstitutionTool::new(validator);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_constitution");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_constitution_tool_execute() {
        let validator = InputValidator::new();
        let tool = ConstitutionTool::new(validator);

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

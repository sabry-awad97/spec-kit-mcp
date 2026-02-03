//! Spec-Kit Clarify Tool
//!
//! Identifies and clarifies ambiguous requirements.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::tools::Tool;
use crate::utils::{validate_file_exists, validate_safe_path};
use crate::validation::InputValidator;

/// Parameters for the speckit_clarify tool
#[derive(Debug, Deserialize, Serialize)]
pub struct ClarifyParams {
    /// Path to specification file
    spec_file: PathBuf,

    /// Specific questions to clarify (optional)
    #[serde(default)]
    questions: Option<Vec<String>>,

    /// Output path for clarifications
    #[serde(default = "default_clarify_path")]
    output_path: PathBuf,
}

fn default_clarify_path() -> PathBuf {
    PathBuf::from("./speckit.clarify")
}

/// Tool for clarifying specifications
pub struct ClarifyTool {
    #[allow(dead_code)] // Will be used for future validation
    validator: InputValidator,
}

impl ClarifyTool {
    /// Create a new clarify tool
    pub fn new(validator: InputValidator) -> Self {
        Self { validator }
    }
}

#[async_trait]
impl Tool for ClarifyTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "speckit_clarify".to_string(),
            description: "Identify underspecified areas in the specification and generate clarification questions".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "spec_file": {
                        "type": "string",
                        "description": "Path to the specification file to analyze"
                    },
                    "questions": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Specific questions to address (optional - will auto-detect if not provided)"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Path where clarifications will be written",
                        "default": "./speckit.clarify"
                    }
                },
                "required": ["spec_file"]
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: ClarifyParams =
            serde_json::from_value(params).context("Failed to parse clarify parameters")?;

        tracing::info!(
            spec_file = %params.spec_file.display(),
            "Analyzing specification for ambiguities"
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

        // Read the specification
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
            "## Task: Clarify Specification\n\n\
            **Specification File**: {}\n\n\
            **Output File**: {}\n\n\
            **User Questions**:\n```\n{}\n```\n\n\
            **Specification Content**:\n```\n{}\n```\n\n\
            ---\n\n\
            ## Instructions\n\n\
            You must now follow the detailed workflow below to identify ambiguities and generate clarifications.\n\
            After generating the content, write it to the output file path above.\n\n\
            **IMPORTANT**: \n\
            - Identify vague terms, missing details, and ambiguous requirements\n\
            - Generate specific, actionable clarification questions\n\
            - Provide context for each question\n\
            - Do NOT write placeholder content\n\n\
            {}",
            params.spec_file.display(),
            safe_path.display(),
            params.questions.as_ref()
                .map(|q| q.join("\n"))
                .unwrap_or_else(|| "(auto-detect ambiguities)".to_string()),
            spec_content,
            crate::templates::CLARIFY_COMMAND
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
    async fn test_clarify_tool_definition() {
        let validator = InputValidator::new();
        let tool = ClarifyTool::new(validator);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_clarify");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_clarify_tool_execute() {
        let validator = InputValidator::new();
        let tool = ClarifyTool::new(validator);

        let dir = tempdir().unwrap();

        // Change to temp directory FIRST
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let spec_file = Path::new("spec.md");
        // Create spec with ambiguities in current directory
        fs::write(spec_file, "We might add OAuth. Performance should be good.")
            .await
            .unwrap();

        let params = json!({
            "spec_file": "spec.md",
            "output_path": "clarify.md"
        });

        let result = tool.execute(params).await.unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Check result - tool should return instructions, not write file
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
    }
}

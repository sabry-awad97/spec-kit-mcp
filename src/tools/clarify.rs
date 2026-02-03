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

        // Return enhanced instructions for AI to follow - DO NOT write file yet
        let message = format!(
            "# ROLE & CONTEXT\n\n\
            You are a **Senior Business Analyst** with 10+ years of experience in requirements clarification and stakeholder communication.\n\n\
            ## Task: Clarify Specification\n\n\
            **Specification File**: {}\n\n\
            **Output File**: {}\n\n\
            **User Questions**:\n```\n{}\n```\n\n\
            **Specification Content**:\n```\n{}\n```\n\n\
            ---\n\n\
            # STRUCTURED THINKING PROTOCOL\n\n\
            Before generating clarifications, complete these steps:\n\n\
            ## [UNDERSTAND]\n\
            - Review the current specification completely\n\
            - Identify all ambiguous terms and vague requirements\n\
            - Note missing decision points and undefined behaviors\n\n\
            ## [ANALYZE]\n\
            - Categorize ambiguities by type (functional, data, UX, non-functional)\n\
            - Assess impact of each ambiguity (blocks implementation vs. nice-to-know)\n\
            - Identify which ambiguities affect multiple areas\n\n\
            ## [STRATEGIZE]\n\
            - Prioritize ambiguities by impact: scope > security > UX > technical\n\
            - Select top 5 most critical clarifications\n\
            - Prepare multiple-choice options with clear trade-offs\n\n\
            ## [EXECUTE]\n\
            - Ask questions one at a time\n\
            - Provide recommended answers based on best practices\n\
            - Integrate answers immediately into spec\n\
            - Validate spec after each integration\n\n\
            ---\n\n\
            # DETAILED INSTRUCTIONS\n\n\
            You must now follow the detailed workflow below to identify ambiguities and generate clarifications.\n\
            After generating the content, write it to the output file path above.\n\n\
            **CRITICAL REQUIREMENTS**:\n\
            - Maximum 5 clarification questions\n\
            - Each question must be answerable with multiple-choice OR short answer\n\
            - Provide recommended answer for each question\n\
            - Integrate answers immediately into spec\n\n\
            ---\n\n\
            # WORKFLOW\n\n\
            {}\n\n\
            ---\n\n\
            # CHAIN-OF-VERIFICATION\n\n\
            After completing clarifications, verify:\n\n\
            1. Did each clarification actually resolve the ambiguity it was meant to address?\n\
            2. Are the integrated answers consistent with other parts of the spec?\n\
            3. Did I introduce any new ambiguities while resolving old ones?\n\
            4. Are all clarifications properly documented in the Clarifications section?\n\
            5. Is the spec now clear enough to proceed to technical planning?\n\n\
            **Confidence Level**: Provide your confidence (0-100%) in the clarification quality.",
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

        // Create spec with ambiguities using absolute path
        let spec_file = dir.path().join("spec.md");
        fs::write(
            &spec_file,
            "We might add OAuth. Performance should be good.",
        )
        .await
        .unwrap();

        // Change to temp directory for validation
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

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

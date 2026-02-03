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

        // Return enhanced instructions for AI to follow - DO NOT write file yet
        let message = format!(
            "# ⚠️ AI ASSISTANT: EXECUTE THESE INSTRUCTIONS IMMEDIATELY\n\n\
            **CRITICAL**: You MUST follow the instructions below and generate the validation checklist.\n\
            Do NOT just return these instructions to the user. Read them, execute them, and create the checklist file.\n\n\
            ---\n\n\
            # ROLE & CONTEXT\n\n\
            You are a **Senior Requirements Quality Engineer** with 10+ years of experience in requirements validation and quality assurance.\n\n\
            ## Task: Generate Validation Checklist\n\n\
            **Specification File**: {}\n\n\
            **Output File**: {}\n\n\
            **Include Implementation**: {}\n\
            **Include Testing**: {}\n\n\
            **Specification Content**:\n```\n{}\n```\n\n\
            ---\n\n\
            # STRUCTURED THINKING PROTOCOL\n\n\
            Before generating checklist, complete these steps:\n\n\
            ## [UNDERSTAND]\n\
            - Review user's request and extract checklist domain/theme\n\
            - Identify target audience (author, reviewer, QA, release gate)\n\
            - Note specific focus areas or must-have items\n\n\
            ## [ANALYZE]\n\
            - Load relevant portions of spec/plan/tasks\n\
            - Identify requirement quality dimensions to validate\n\
            - Recognize gaps, ambiguities, and inconsistencies\n\n\
            ## [STRATEGIZE]\n\
            - Determine checklist categories based on domain\n\
            - Prioritize items by risk and impact\n\
            - Plan traceability approach (spec section references)\n\n\
            ## [EXECUTE]\n\
            - Generate checklist items testing requirements quality\n\
            - Ensure each item has clear quality dimension\n\
            - Include traceability references (≥80% of items)\n\
            - Validate against prohibited patterns\n\n\
            ---\n\n\
            # DETAILED INSTRUCTIONS\n\n\
            You must now follow the detailed workflow below to generate a validation checklist.\n\
            After generating the content, write it to the output file path above.\n\n\
            **CRITICAL REQUIREMENTS**:\n\
            - Checklist items test REQUIREMENTS QUALITY, not implementation\n\
            - Each item must be a question about what's written in the spec\n\
            - Include quality dimension in brackets [Completeness/Clarity/etc.]\n\
            - ≥80% of items must have traceability references\n\n\
            ---\n\n\
            # WORKFLOW\n\n\
            {}\n\n\
            ---\n\n\
            # CHAIN-OF-VERIFICATION\n\n\
            After generating checklist, verify:\n\n\
            1. Does every item test requirements quality, not implementation behavior?\n\
            2. Do ≥80% of items include traceability references?\n\
            3. Are items organized by clear quality dimensions?\n\
            4. Have I avoided prohibited patterns (Verify, Test, Confirm + behavior)?\n\
            5. Is the checklist focused on high-impact quality issues?\n\n\
            **Confidence Level**: Provide your confidence (0-100%) in the checklist quality.",
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

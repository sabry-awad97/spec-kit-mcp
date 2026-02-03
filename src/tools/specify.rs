//! Spec-Kit Specify Tool
//!
//! Defines requirements and user stories.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::tools::Tool;
use crate::utils::{validate_project_initialized_for_path, validate_safe_path};
use crate::validation::InputValidator;

/// Parameters for the speckit_specify tool
#[derive(Debug, Deserialize, Serialize)]
pub struct SpecifyParams {
    /// Requirements to specify
    requirements: String,

    /// User stories (optional)
    #[serde(default)]
    user_stories: Option<String>,

    /// Output path for specification file
    #[serde(default = "default_specify_path")]
    output_path: PathBuf,

    /// Output format
    #[serde(default = "default_format")]
    format: String,
}

fn default_specify_path() -> PathBuf {
    PathBuf::from("./speckit.specify")
}

fn default_format() -> String {
    "markdown".to_string()
}

/// Tool for creating specifications
pub struct SpecifyTool {
    validator: InputValidator,
}

impl SpecifyTool {
    /// Create a new specify tool
    pub fn new(validator: InputValidator) -> Self {
        Self { validator }
    }
}

#[async_trait]
impl Tool for SpecifyTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "speckit_specify".to_string(),
            description: "Define what you want to build - requirements, user stories, and acceptance criteria".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "requirements": {
                        "type": "string",
                        "description": "The requirements to specify. Can include features, constraints, user needs, etc."
                    },
                    "user_stories": {
                        "type": "string",
                        "description": "Optional user stories in 'As a... I want... So that...' format"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Path where the specification file will be written",
                        "default": "./speckit.specify"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["markdown", "yaml", "json"],
                        "default": "markdown",
                        "description": "Output format for the specification"
                    }
                },
                "required": ["requirements"]
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: SpecifyParams =
            serde_json::from_value(params).context("Failed to parse specify parameters")?;

        tracing::info!(
            output_path = %params.output_path.display(),
            format = %params.format,
            "Creating specification"
        );

        // Validate requirements
        if let Err(e) = self.validator.validate_requirements(&params.requirements) {
            return Ok(ToolResult {
                content: vec![ContentBlock::text(format!(
                    "Invalid requirements: {}\n\nSuggestion: {}",
                    e, "Ensure requirements are not empty and within size limits"
                ))],
                is_error: Some(true),
            });
        }

        // Validate user stories if provided
        if let Some(ref stories) = params.user_stories {
            if let Err(e) = self.validator.validate_user_stories(stories) {
                return Ok(ToolResult {
                    content: vec![ContentBlock::text(format!(
                        "Invalid user stories: {}\n\nSuggestion: {}",
                        e, "Ensure user stories are within size limits"
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
            You are a **Senior Requirements Engineer** with 10+ years of experience in software specification.\n\n\
            ## Task: Create Specification\n\n\
            **Output File**: {}\n\n\
            **User Requirements**:\n```\n{}\n```\n\n\
            **User Stories**:\n```\n{}\n```\n\n\
            ---\n\n\
            # STRUCTURED THINKING PROTOCOL\n\n\
            Before generating the specification, complete these steps:\n\n\
            ## [UNDERSTAND]\n\
            - Restate the feature request in your own words\n\
            - Identify the core user problem being solved\n\
            - Extract key actors, actions, and expected outcomes\n\n\
            ## [ANALYZE]\n\
            - Break down into functional components\n\
            - Identify implicit assumptions that need validation\n\
            - Recognize potential edge cases and failure scenarios\n\n\
            ## [STRATEGIZE]\n\
            - Determine reasonable defaults for underspecified aspects\n\
            - Prioritize clarification questions by impact\n\
            - Plan specification structure based on complexity\n\n\
            ## [EXECUTE]\n\
            - Generate specification following validated strategy\n\
            - Apply quality validation checkpoints\n\
            - Provide confidence assessment\n\n\
            ---\n\n\
            # DETAILED INSTRUCTIONS\n\n\
            You must now follow the detailed workflow below to generate a complete specification.\n\
            After generating the content, write it to the output file path above.\n\n\
            **CRITICAL REQUIREMENTS**:\n\
            - Do NOT write placeholder content\n\
            - Generate fully populated content following the instructions\n\
            - Focus on WHAT users need, not HOW to implement\n\
            - Make requirements testable and measurable\n\
            - Document assumptions clearly\n\n\
            ---\n\n\
            # WORKFLOW\n\n\
            {}\n\n\
            ---\n\n\
            # CHAIN-OF-VERIFICATION\n\n\
            After generating the specification, verify:\n\n\
            1. Are all user-facing features described without mentioning specific technologies?\n\
            2. Can each functional requirement be tested without knowing the implementation?\n\
            3. Are success criteria measurable with specific metrics?\n\
            4. Have I documented all assumptions made?\n\
            5. Are there any remaining ambiguities that would block planning?\n\n\
            **Confidence Level**: Provide your confidence (0-100%) in the specification quality.\n\n\
            **Key Assumptions**: List critical assumptions made.\n\n\
            **What Would Change This**: Factors that would require spec revision.",
            safe_path.display(),
            params.requirements,
            params.user_stories.as_deref().unwrap_or("(none provided)"),
            crate::templates::SPECIFY_COMMAND
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
    async fn test_specify_tool_definition() {
        let validator = InputValidator::new();
        let tool = SpecifyTool::new(validator);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_specify");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_specify_tool_execute() {
        let validator = InputValidator::new();
        let tool = SpecifyTool::new(validator);

        let dir = tempdir().unwrap();

        // Create .specify directory to simulate initialized project
        let specify_dir = dir.path().join(".specify");
        tokio::fs::create_dir(&specify_dir).await.unwrap();

        let params = json!({
            "requirements": "User authentication system with OAuth2 support",
            "user_stories": "As a user, I want to login with Google, so that I don't need another password",
            "output_path": "specification.md"
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
        // assert!(!output_path.exists());
    }
}

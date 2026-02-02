//! Spec-Kit Analyze Tool
//!
//! Analyzes cross-artifact consistency and coverage.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::speckit::SpecKitCli;
use crate::tools::Tool;
use crate::utils::validate_safe_path;

/// Parameters for the speckit_analyze tool
#[derive(Debug, Deserialize, Serialize)]
pub struct AnalyzeParams {
    /// Path to project directory
    project_path: PathBuf,

    /// Check consistency across artifacts
    #[serde(default = "default_true")]
    check_consistency: bool,

    /// Check coverage of requirements
    #[serde(default = "default_true")]
    check_coverage: bool,

    /// Output path for analysis report
    #[serde(default = "default_analyze_path")]
    output_path: PathBuf,
}

fn default_true() -> bool {
    true
}

fn default_analyze_path() -> PathBuf {
    PathBuf::from("./speckit.analyze")
}

/// Tool for analyzing spec-kit artifacts
pub struct AnalyzeTool {
    #[allow(dead_code)]
    cli: SpecKitCli,
}

impl AnalyzeTool {
    /// Create a new analyze tool
    pub fn new(cli: SpecKitCli) -> Self {
        Self { cli }
    }
}

#[async_trait]
impl Tool for AnalyzeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "speckit_analyze".to_string(),
            description: "Analyze cross-artifact consistency and coverage - ensures constitution, specs, plans, and tasks are aligned".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory containing spec-kit artifacts"
                    },
                    "check_consistency": {
                        "type": "boolean",
                        "default": true,
                        "description": "Check if artifacts are consistent with each other"
                    },
                    "check_coverage": {
                        "type": "boolean",
                        "default": true,
                        "description": "Check if all requirements are covered in plan/tasks"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Path where analysis report will be written",
                        "default": "./speckit.analyze"
                    }
                },
                "required": ["project_path"]
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: AnalyzeParams =
            serde_json::from_value(params).context("Failed to parse analyze parameters")?;

        tracing::info!(
            project_path = %params.project_path.display(),
            "Analyzing project artifacts"
        );

        // Validate output path is safe
        let safe_output_path = match validate_safe_path(&params.output_path) {
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

        // Validate project path exists
        if !params.project_path.exists() {
            return Ok(ToolResult {
                content: vec![ContentBlock::text(format!(
                    "Project path does not exist: {}\n\n\
                    Please provide a valid project directory.",
                    params.project_path.display()
                ))],
                is_error: Some(true),
            });
        }

        // Check for artifacts
        let artifacts = vec![
            ("Constitution", "speckit.constitution"),
            ("Specification", "speckit.specify"),
            ("Plan", "speckit.plan"),
            ("Tasks", "speckit.tasks"),
        ];

        let mut artifact_info = String::new();
        let mut found_count = 0;

        for (name, filename) in &artifacts {
            let path = params.project_path.join(filename);
            if path.exists() {
                artifact_info.push_str(&format!("✓ {} found at {}\n", name, path.display()));
                found_count += 1;
            } else {
                artifact_info.push_str(&format!(
                    "✗ {} missing (expected at {})\n",
                    name,
                    path.display()
                ));
            }
        }

        // Return instructions for AI to follow - DO NOT write file yet
        let message = format!(
            "## Task: Analyze Project Artifacts\n\n\
            **Project Path**: {}\n\n\
            **Output File**: {}\n\n\
            **Check Consistency**: {}\n\
            **Check Coverage**: {}\n\n\
            **Artifacts Found**: {}/{}\n\n\
            {}\n\n\
            ---\n\n\
            ## Instructions\n\n\
            You must now follow the detailed workflow below to analyze cross-artifact consistency.\n\
            After generating the content, write it to the output file path above.\n\n\
            **IMPORTANT**: \n\
            - Read all available artifacts\n\
            - Check for consistency between constitution, spec, plan, and tasks\n\
            - Verify all requirements are covered\n\
            - Identify gaps and inconsistencies\n\
            - Do NOT write placeholder content\n\n\
            {}",
            params.project_path.display(),
            safe_output_path.display(),
            params.check_consistency,
            params.check_coverage,
            found_count,
            artifacts.len(),
            artifact_info,
            crate::templates::ANALYZE_COMMAND
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
    async fn test_analyze_tool_definition() {
        let cli = SpecKitCli::new();
        let tool = AnalyzeTool::new(cli);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_analyze");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_tool_execute() {
        let cli = SpecKitCli::new_test_mode();
        let tool = AnalyzeTool::new(cli);

        let dir = tempdir().unwrap();

        // Create some artifacts
        fs::write(dir.path().join("speckit.constitution"), "Principles")
            .await
            .unwrap();
        fs::write(dir.path().join("speckit.specify"), "Requirements")
            .await
            .unwrap();

        let params = json!({
            "project_path": ".",
            "check_consistency": true,
            "check_coverage": true
        });

        // Change to temp directory for test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = tool.execute(params).await.unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Check result - tool should return instructions, not write file
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
    }
}

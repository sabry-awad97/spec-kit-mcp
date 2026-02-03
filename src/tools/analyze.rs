//! Spec-Kit Analyze Tool
//!
//! Analyzes cross-artifact consistency and coverage.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::tools::Tool;
use crate::utils::validate_safe_path;
use crate::validation::InputValidator;

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
    #[allow(dead_code)] // Will be used for future validation
    validator: InputValidator,
}

impl AnalyzeTool {
    /// Create a new analyze tool
    pub fn new(validator: InputValidator) -> Self {
        Self { validator }
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

        // Return enhanced instructions for AI to follow - DO NOT write file yet
        let message = format!(
            "# ROLE & CONTEXT\n\n\
            You are a **Senior Quality Assurance Architect** with 12+ years of experience in requirements validation and consistency analysis.\n\n\
            ## Task: Analyze Project Artifacts\n\n\
            **Project Path**: {}\n\n\
            **Output File**: {}\n\n\
            **Check Consistency**: {}\n\
            **Check Coverage**: {}\n\n\
            **Artifacts Found**: {}/{}\n\n\
            {}\n\n\
            ---\n\n\
            # STRUCTURED THINKING PROTOCOL\n\n\
            Before performing analysis, complete these steps:\n\n\
            ## [UNDERSTAND]\n\
            - Review scope of analysis (spec, plan, tasks)\n\
            - Identify constitution principles to validate against\n\
            - Note expected relationships between artifacts\n\n\
            ## [ANALYZE]\n\
            - Load and parse all three artifacts efficiently\n\
            - Build semantic models of requirements, architecture, tasks\n\
            - Identify relationships and dependencies\n\
            - Detect patterns of inconsistency or gaps\n\n\
            ## [STRATEGIZE]\n\
            - Prioritize findings by severity (CRITICAL > HIGH > MEDIUM > LOW)\n\
            - Focus on high-signal issues that block implementation\n\
            - Plan remediation recommendations\n\n\
            ## [EXECUTE]\n\
            - Generate structured analysis report\n\
            - Provide specific examples with line references\n\
            - Offer concrete remediation suggestions\n\
            - Deliver actionable recommendations\n\n\
            ---\n\n\
            # DETAILED INSTRUCTIONS\n\n\
            You must now follow the detailed workflow below to analyze cross-artifact consistency.\n\
            After generating the content, write it to the output file path above.\n\n\
            **CRITICAL REQUIREMENTS**:\n\
            - STRICTLY READ-ONLY (do not modify any files)\n\
            - Constitution violations are automatically CRITICAL\n\
            - Limit findings to 50 (prioritize by severity)\n\
            - Provide specific locations and actionable recommendations\n\n\
            ---\n\n\
            # WORKFLOW\n\n\
            {}\n\n\
            ---\n\n\
            # CHAIN-OF-VERIFICATION\n\n\
            After completing analysis, verify:\n\n\
            1. Did I correctly identify all constitution violations?\n\
            2. Are my severity assignments consistent and justified?\n\
            3. Have I provided actionable recommendations for each finding?\n\
            4. Did I miss any obvious coverage gaps or inconsistencies?\n\
            5. Is my analysis focused on high-impact issues vs. nitpicking?\n\n\
            **Confidence Level**: Provide your confidence (0-100%) in the analysis quality.",
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
        let validator = InputValidator::new();
        let tool = AnalyzeTool::new(validator);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_analyze");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_tool_execute() {
        let validator = InputValidator::new();
        let tool = AnalyzeTool::new(validator);

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

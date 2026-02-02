//! Spec-Kit Check Tool
//!
//! Validates that required tools are installed.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::speckit::SpecKitCli;
use crate::tools::Tool;

/// Parameters for the speckit_check tool
#[derive(Debug, Deserialize, Serialize)]
pub struct CheckParams {
    /// Check for spec-kit CLI
    #[serde(default = "default_true")]
    check_speckit: bool,

    /// Check for git
    #[serde(default = "default_true")]
    check_git: bool,

    /// Check for common AI coding assistants
    #[serde(default = "default_true")]
    check_ai_tools: bool,
}

fn default_true() -> bool {
    true
}

impl Default for CheckParams {
    fn default() -> Self {
        Self {
            check_speckit: true,
            check_git: true,
            check_ai_tools: true,
        }
    }
}

/// Tool for checking required tool installations
pub struct CheckTool {
    cli: SpecKitCli,
}

impl CheckTool {
    /// Create a new check tool
    pub fn new(cli: SpecKitCli) -> Self {
        Self { cli }
    }
}

#[async_trait]
impl Tool for CheckTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "speckit_check".to_string(),
            description: "Validate that required tools are installed for spec-kit development"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "check_speckit": {
                        "type": "boolean",
                        "default": true,
                        "description": "Check if spec-kit CLI is installed"
                    },
                    "check_git": {
                        "type": "boolean",
                        "default": true,
                        "description": "Check if git is installed"
                    },
                    "check_ai_tools": {
                        "type": "boolean",
                        "default": true,
                        "description": "Check for AI coding assistants (claude, cursor, etc.)"
                    }
                },
                "required": []
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let _params: CheckParams =
            if params.is_null() || params.as_object().is_some_and(|o| o.is_empty()) {
                CheckParams::default()
            } else {
                serde_json::from_value(params).context("Failed to parse check parameters")?
            };

        tracing::info!("Checking tool installations");

        // Execute the spec-kit check command
        let result = self.cli.check().await?;
        let is_success = result.is_success();

        // The spec-kit check command provides its own formatted output
        Ok(ToolResult {
            content: vec![ContentBlock::text(result.stdout)],
            is_error: Some(!is_success),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_tool_definition() {
        let cli = SpecKitCli::new();
        let tool = CheckTool::new(cli);
        let def = tool.definition();

        assert_eq!(def.name, "speckit_check");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_check_tool_execute() {
        let cli = SpecKitCli::new_test_mode();
        let tool = CheckTool::new(cli);

        // Test with default params
        let result = tool.execute(json!({})).await.unwrap();
        assert!(!result.content.is_empty());

        // Test with specific params
        let params = json!({
            "check_speckit": true,
            "check_git": true,
            "check_ai_tools": false
        });

        let result = tool.execute(params).await.unwrap();
        assert!(!result.content.is_empty());
    }
}

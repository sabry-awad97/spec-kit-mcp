//! Spec-Kit Check Tool
//!
//! Validates that required tools are installed.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
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
#[derive(Default)]
pub struct CheckTool {}

impl CheckTool {
    /// Create a new check tool
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a command is available
    async fn check_command(&self, command: &str) -> bool {
        async_process::Command::new(command)
            .arg("--version")
            .stdout(async_process::Stdio::null())
            .stderr(async_process::Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
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
        let params: CheckParams =
            if params.is_null() || params.as_object().is_some_and(|o| o.is_empty()) {
                CheckParams::default()
            } else {
                serde_json::from_value(params).context("Failed to parse check parameters")?
            };

        tracing::info!("Checking tool installations");

        let mut output = String::from("# Tool Installation Check\n\n");

        // Check git
        if params.check_git {
            let git_available = self.check_command("git").await;
            if git_available {
                output.push_str("✓ git is installed\n");
            } else {
                output.push_str("✗ git is NOT installed\n");
                output.push_str("  Install from: https://git-scm.com/downloads\n");
            }
        }

        // Check spec-kit (this MCP server provides spec-kit functionality)
        if params.check_speckit {
            output.push_str(
                "\n✓ spec-kit MCP server is running (provides all spec-kit functionality)\n",
            );
        }

        // Check AI tools
        if params.check_ai_tools {
            output.push_str("\n## AI Coding Assistants\n\n");

            let ai_tools = vec![
                ("claude", "Claude Desktop"),
                ("cursor", "Cursor"),
                ("code", "VS Code"),
            ];

            for (cmd, name) in ai_tools {
                let available = self.check_command(cmd).await;
                if available {
                    output.push_str(&format!("✓ {} is installed\n", name));
                } else {
                    output.push_str(&format!("○ {} not found (optional)\n", name));
                }
            }
        }

        output.push_str("\n## Summary\n\n");
        output.push_str("The spec-kit MCP server provides all core functionality.\n");
        output.push_str("Git is recommended for version control.\n");
        output.push_str("AI coding assistants enhance the development experience.\n");

        Ok(ToolResult {
            content: vec![ContentBlock::text(output)],
            is_error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_tool_definition() {
        let tool = CheckTool::new();
        let def = tool.definition();

        assert_eq!(def.name, "speckit_check");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_check_tool_execute() {
        let tool = CheckTool::new();

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

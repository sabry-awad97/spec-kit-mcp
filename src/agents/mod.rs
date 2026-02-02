//! Agent Configuration System
//!
//! Manages AI assistant configurations for spec-kit projects.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for an AI assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Display name of the agent
    pub name: String,

    /// Folder name for agent-specific files
    pub folder: String,

    /// Whether the agent requires a CLI tool
    pub requires_cli: bool,

    /// Installation URL for the agent
    pub install_url: String,

    /// Command to check if agent is installed
    pub check_command: Option<String>,
}

/// Get all supported agent configurations
pub fn get_agent_configs() -> HashMap<String, AgentConfig> {
    let mut configs = HashMap::new();

    // Claude Desktop
    configs.insert(
        "claude".to_string(),
        AgentConfig {
            name: "Claude Desktop".to_string(),
            folder: ".claude".to_string(),
            requires_cli: false,
            install_url: "https://claude.ai/download".to_string(),
            check_command: None,
        },
    );

    // GitHub Copilot
    configs.insert(
        "copilot".to_string(),
        AgentConfig {
            name: "GitHub Copilot".to_string(),
            folder: ".github/copilot".to_string(),
            requires_cli: false,
            install_url: "https://github.com/features/copilot".to_string(),
            check_command: None,
        },
    );

    // Cursor
    configs.insert(
        "cursor".to_string(),
        AgentConfig {
            name: "Cursor".to_string(),
            folder: ".cursor".to_string(),
            requires_cli: false,
            install_url: "https://cursor.sh".to_string(),
            check_command: Some("cursor".to_string()),
        },
    );

    // Windsurf
    configs.insert(
        "windsurf".to_string(),
        AgentConfig {
            name: "Windsurf".to_string(),
            folder: ".windsurf".to_string(),
            requires_cli: false,
            install_url: "https://codeium.com/windsurf".to_string(),
            check_command: None,
        },
    );

    // Gemini
    configs.insert(
        "gemini".to_string(),
        AgentConfig {
            name: "Google Gemini".to_string(),
            folder: ".gemini".to_string(),
            requires_cli: false,
            install_url: "https://gemini.google.com".to_string(),
            check_command: None,
        },
    );

    // VS Code
    configs.insert(
        "vscode".to_string(),
        AgentConfig {
            name: "VS Code".to_string(),
            folder: ".vscode".to_string(),
            requires_cli: false,
            install_url: "https://code.visualstudio.com".to_string(),
            check_command: Some("code".to_string()),
        },
    );

    // Qwen
    configs.insert(
        "qwen".to_string(),
        AgentConfig {
            name: "Qwen".to_string(),
            folder: ".qwen".to_string(),
            requires_cli: false,
            install_url: "https://qwenlm.github.io".to_string(),
            check_command: None,
        },
    );

    // OpenCode
    configs.insert(
        "opencode".to_string(),
        AgentConfig {
            name: "OpenCode".to_string(),
            folder: ".opencode".to_string(),
            requires_cli: false,
            install_url: "https://opencode.ai".to_string(),
            check_command: None,
        },
    );

    // Codex
    configs.insert(
        "codex".to_string(),
        AgentConfig {
            name: "OpenAI Codex".to_string(),
            folder: ".codex".to_string(),
            requires_cli: false,
            install_url: "https://openai.com/codex".to_string(),
            check_command: None,
        },
    );

    // Kilocode
    configs.insert(
        "kilocode".to_string(),
        AgentConfig {
            name: "Kilocode".to_string(),
            folder: ".kilocode".to_string(),
            requires_cli: false,
            install_url: "https://kilocode.ai".to_string(),
            check_command: None,
        },
    );

    // Auggie
    configs.insert(
        "auggie".to_string(),
        AgentConfig {
            name: "Auggie".to_string(),
            folder: ".auggie".to_string(),
            requires_cli: false,
            install_url: "https://auggie.ai".to_string(),
            check_command: None,
        },
    );

    // CodeBuddy
    configs.insert(
        "codebuddy".to_string(),
        AgentConfig {
            name: "CodeBuddy".to_string(),
            folder: ".codebuddy".to_string(),
            requires_cli: false,
            install_url: "https://codebuddy.ai".to_string(),
            check_command: None,
        },
    );

    // AMP
    configs.insert(
        "amp".to_string(),
        AgentConfig {
            name: "AMP".to_string(),
            folder: ".amp".to_string(),
            requires_cli: false,
            install_url: "https://amp.dev".to_string(),
            check_command: None,
        },
    );

    // Shai
    configs.insert(
        "shai".to_string(),
        AgentConfig {
            name: "Shai".to_string(),
            folder: ".shai".to_string(),
            requires_cli: false,
            install_url: "https://shai.ai".to_string(),
            check_command: None,
        },
    );

    // Q
    configs.insert(
        "q".to_string(),
        AgentConfig {
            name: "Q".to_string(),
            folder: ".q".to_string(),
            requires_cli: false,
            install_url: "https://q.ai".to_string(),
            check_command: None,
        },
    );

    // Bob
    configs.insert(
        "bob".to_string(),
        AgentConfig {
            name: "Bob".to_string(),
            folder: ".bob".to_string(),
            requires_cli: false,
            install_url: "https://bob.ai".to_string(),
            check_command: None,
        },
    );

    // Qoder
    configs.insert(
        "qoder".to_string(),
        AgentConfig {
            name: "Qoder".to_string(),
            folder: ".qoder".to_string(),
            requires_cli: false,
            install_url: "https://qoder.ai".to_string(),
            check_command: None,
        },
    );

    // Kiro
    configs.insert(
        "kiro".to_string(),
        AgentConfig {
            name: "Kiro".to_string(),
            folder: ".kiro".to_string(),
            requires_cli: false,
            install_url: "https://kiro.dev/".to_string(),
            check_command: None,
        },
    );

    configs
}

/// Get a specific agent configuration
pub fn get_agent_config(agent_id: &str) -> Option<AgentConfig> {
    get_agent_configs().get(agent_id).cloned()
}

/// Get list of all supported agent IDs
pub fn get_agent_ids() -> Vec<String> {
    let mut ids: Vec<String> = get_agent_configs().keys().cloned().collect();
    ids.sort();
    ids
}

/// Check if an agent is installed (if it has a check command)
pub async fn is_agent_installed(agent_id: &str) -> bool {
    if let Some(config) = get_agent_config(agent_id) {
        if let Some(check_cmd) = config.check_command {
            return async_process::Command::new(&check_cmd)
                .arg("--version")
                .stdout(async_process::Stdio::null())
                .stderr(async_process::Stdio::null())
                .status()
                .await
                .map(|s| s.success())
                .unwrap_or(false);
        }
    }
    true // If no check command, assume available
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_agent_configs() {
        let configs = get_agent_configs();
        assert!(!configs.is_empty());
        assert!(configs.contains_key("claude"));
        assert!(configs.contains_key("copilot"));
        assert!(configs.contains_key("cursor"));
        assert!(configs.contains_key("kiro"));
    }

    #[test]
    fn test_get_agent_config() {
        let config = get_agent_config("claude").unwrap();
        assert_eq!(config.name, "Claude Desktop");
        assert_eq!(config.folder, ".claude");
    }

    #[test]
    fn test_get_agent_ids() {
        let ids = get_agent_ids();
        assert!(!ids.is_empty());
        assert!(ids.contains(&"claude".to_string()));
    }

    #[test]
    fn test_invalid_agent() {
        let config = get_agent_config("nonexistent");
        assert!(config.is_none());
    }
}

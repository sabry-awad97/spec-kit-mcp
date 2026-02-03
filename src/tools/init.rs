//! Spec-Kit Init Tool
//!
//! Initializes a new spec-kit project by downloading and extracting the template.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::mcp::types::{ContentBlock, ToolDefinition, ToolResult};
use crate::tools::Tool;

/// Parameters for the speckit_init tool
#[derive(Debug, Deserialize, Serialize)]
pub struct InitParams {
    /// Project name
    project_name: String,

    /// Project path (defaults to current directory)
    #[serde(default = "default_project_path")]
    project_path: PathBuf,

    /// AI assistant to use (claude, copilot, cursor, etc.)
    #[serde(default)]
    ai_assistant: Option<String>,

    /// Script type (sh or ps)
    #[serde(default)]
    script_type: Option<String>,

    /// Skip git initialization
    #[serde(default)]
    no_git: bool,
}

fn default_project_path() -> PathBuf {
    PathBuf::from(".")
}

/// Tool for initializing spec-kit projects
#[derive(Default)]
pub struct InitTool {}

impl InitTool {
    /// Create a new init tool
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize project with native Rust implementation (using embedded templates)
    async fn init_native(
        &self,
        project_name: &str,
        project_path: &Path,
        params: &InitParams,
    ) -> Result<String> {
        // Determine actual project path
        let actual_path = if project_path == Path::new(".") || project_path == Path::new("") {
            PathBuf::from(project_name)
        } else {
            project_path.to_path_buf()
        };

        // Check if directory already exists
        if actual_path.exists() {
            return Err(anyhow!(
                "Directory '{}' already exists. Please choose a different project name or remove the existing directory.",
                actual_path.display()
            ));
        }

        tracing::info!("Creating project structure with embedded templates");

        // Create project directory
        fs::create_dir_all(&actual_path).context("Failed to create project directory")?;

        // Create .specify directory structure
        let specify_dir = actual_path.join(".specify");
        fs::create_dir_all(&specify_dir).context("Failed to create .specify directory")?;

        // Create subdirectories
        fs::create_dir_all(specify_dir.join("memory"))
            .context("Failed to create .specify/memory directory")?;
        fs::create_dir_all(specify_dir.join("templates"))
            .context("Failed to create .specify/templates directory")?;

        // Write embedded templates to .specify/templates/
        fs::write(
            specify_dir.join("templates/constitution.md"),
            crate::templates::CONSTITUTION_TEMPLATE,
        )?;
        fs::write(
            specify_dir.join("templates/spec-template.md"),
            crate::templates::SPEC_TEMPLATE,
        )?;
        fs::write(
            specify_dir.join("templates/plan-template.md"),
            crate::templates::PLAN_TEMPLATE,
        )?;
        fs::write(
            specify_dir.join("templates/tasks-template.md"),
            crate::templates::TASKS_TEMPLATE,
        )?;
        fs::write(
            specify_dir.join("templates/checklist-template.md"),
            crate::templates::CHECKLIST_TEMPLATE,
        )?;

        // Create initial constitution in memory/
        let constitution_content = format!(
            "# {} - Project Constitution\n\n\
            ## Core Principles\n\n\
            1. [Add your core principles here]\n\n\
            ## Technical Constraints\n\n\
            - [Add technical constraints here]\n\n\
            ## Development Standards\n\n\
            - [Add development standards here]\n",
            project_name
        );
        fs::write(
            specify_dir.join("memory/constitution.md"),
            constitution_content,
        )?;

        tracing::info!("Created .specify directory structure with embedded templates");
        if !specify_dir.exists() {
            return Err(anyhow!(
                "Template extraction failed: .specify directory not found"
            ));
        }

        // Setup agent configuration
        let agent_id = params.ai_assistant.as_deref().unwrap_or("claude");
        let agent_config = crate::agents::get_agent_config(agent_id)
            .ok_or_else(|| anyhow!("Unknown agent: {}", agent_id))?;
        self.setup_agent_config(&actual_path, agent_id)?;

        // Setup script permissions (Unix only)
        #[cfg(unix)]
        self.ensure_scripts_executable(&actual_path)?;

        // Initialize git repository (if not disabled)
        let git_initialized = if !params.no_git {
            self.init_git_repo(&actual_path).await.is_ok()
        } else {
            false
        };

        // Build success message with structured guidance
        let mut message = String::new();

        message.push_str("# Project Initialization Complete ✓\n\n");
        message.push_str("---\n\n");

        message.push_str("## Project Details\n\n");
        message.push_str(&format!("- **Project Name**: {}\n", project_name));
        message.push_str(&format!("- **Location**: {}\n", actual_path.display()));
        message.push_str(&format!("- **AI Assistant**: {}\n", agent_id));
        message.push_str(&format!(
            "- **Script Type**: {}\n",
            params
                .script_type
                .as_deref()
                .unwrap_or(if cfg!(windows) { "ps" } else { "sh" })
        ));
        message.push_str(&format!(
            "- **Git Repository**: {}\n\n",
            if git_initialized {
                "Initialized"
            } else {
                "Skipped"
            }
        ));

        message.push_str("## Created Structure\n\n");
        message.push_str("```\n");
        message.push_str(&format!("{}/\n", project_name));
        message.push_str("├── .specify/\n");
        message.push_str("│   ├── memory/\n");
        message.push_str("│   │   └── constitution.md    # Project principles and standards\n");
        message.push_str("│   └── templates/\n");
        message.push_str("│       ├── constitution.md    # Constitution template\n");
        message.push_str("│       ├── spec-template.md   # Specification template\n");
        message.push_str("│       ├── plan-template.md   # Technical plan template\n");
        message.push_str("│       ├── tasks-template.md  # Task list template\n");
        message.push_str("│       └── checklist-template.md  # Quality checklist template\n");
        message.push_str(&format!(
            "└── {}/ # Agent configuration\n",
            agent_config.folder
        ));
        if git_initialized {
            message.push_str("    └── .git/              # Git repository\n");
        }
        message.push_str("```\n\n");

        message.push_str("---\n\n");
        message.push_str("## Spec-Kit Development Workflow\n\n");
        message.push_str("Follow this structured approach to build your feature:\n\n");

        message.push_str("### Phase 1: Foundation (Define What to Build)\n\n");
        message.push_str(&format!(
            "1. **Navigate to project**: `cd {}`\n\n",
            project_name
        ));
        message.push_str("2. **Define project principles** (Optional but recommended)\n");
        message.push_str("   - Tool: `speckit_constitution`\n");
        message.push_str("   - Purpose: Establish core principles, technical constraints, and development standards\n");
        message.push_str("   - Output: `.specify/memory/constitution.md`\n");
        message.push_str(
            "   - Example: \"Focus on simplicity, prioritize security, use TypeScript\"\n\n",
        );

        message.push_str("3. **Specify requirements**\n");
        message.push_str("   - Tool: `speckit_specify`\n");
        message.push_str("   - Purpose: Define WHAT you want to build (features, user stories, acceptance criteria)\n");
        message.push_str("   - Output: `speckit.specify`\n");
        message
            .push_str("   - Focus: User needs and business requirements (technology-agnostic)\n\n");

        message.push_str("4. **Clarify ambiguities** (If needed)\n");
        message.push_str("   - Tool: `speckit_clarify`\n");
        message.push_str(
            "   - Purpose: Identify underspecified areas and generate clarification questions\n",
        );
        message.push_str("   - Output: `speckit.clarify`\n");
        message.push_str("   - Use when: Specification has unclear or ambiguous requirements\n\n");

        message.push_str("### Phase 2: Planning (Define How to Build)\n\n");
        message.push_str("5. **Create technical plan**\n");
        message.push_str("   - Tool: `speckit_plan`\n");
        message.push_str(
            "   - Purpose: Design HOW to implement (architecture, tech stack, approach)\n",
        );
        message.push_str("   - Output: `speckit.plan`\n");
        message.push_str(
            "   - Includes: Technology choices, system design, implementation strategy\n\n",
        );

        message.push_str("6. **Generate task list**\n");
        message.push_str("   - Tool: `speckit_tasks`\n");
        message.push_str("   - Purpose: Break down plan into actionable, ordered tasks\n");
        message.push_str("   - Output: `speckit.tasks`\n");
        message.push_str("   - Result: Prioritized task list with dependencies and estimates\n\n");

        message.push_str("7. **Generate validation checklist** (Optional)\n");
        message.push_str("   - Tool: `speckit_checklist`\n");
        message.push_str("   - Purpose: Create quality validation checklist from specification\n");
        message.push_str("   - Output: `speckit.checklist`\n");
        message
            .push_str("   - Use for: Ensuring all requirements are met during implementation\n\n");

        message.push_str("### Phase 3: Execution (Build It)\n\n");
        message.push_str("8. **Execute implementation**\n");
        message.push_str("   - Tool: `speckit_implement`\n");
        message.push_str("   - Purpose: Generate code and documentation following the task list\n");
        message.push_str("   - Output: Source code in specified directory (default: `./src`)\n");
        message.push_str("   - Process: Implements tasks in order, validates each phase\n\n");

        message.push_str("### Phase 4: Validation (Verify Quality)\n\n");
        message.push_str("9. **Analyze consistency**\n");
        message.push_str("   - Tool: `speckit_analyze`\n");
        message.push_str(
            "   - Purpose: Verify alignment between constitution, specs, plans, and tasks\n",
        );
        message.push_str("   - Output: `speckit.analyze`\n");
        message.push_str("   - Checks: Cross-artifact consistency and requirement coverage\n\n");

        message.push_str("---\n\n");
        message.push_str("## Quick Start Example\n\n");
        message.push_str("```bash\n");
        message.push_str(&format!(
            "# Navigate to your project\ncd {}\n\n",
            project_name
        ));
        message.push_str("# Define project principles (optional)\n");
        message.push_str("# Use speckit_constitution tool with your principles\n\n");
        message.push_str("# Specify what you want to build\n");
        message.push_str("# Use speckit_specify tool with your requirements\n");
        message.push_str(
            "# Example: \"Build a user authentication system with email/password login\"\n\n",
        );
        message.push_str("# Create technical plan\n");
        message.push_str("# Use speckit_plan tool referencing your specification\n\n");
        message.push_str("# Generate actionable tasks\n");
        message.push_str("# Use speckit_tasks tool referencing your plan\n\n");
        message.push_str("# Execute implementation\n");
        message.push_str("# Use speckit_implement tool referencing your tasks\n");
        message.push_str("```\n\n");

        message.push_str("---\n\n");
        message.push_str("## Key Principles\n\n");
        message.push_str("- **Specification First**: Define WHAT before HOW\n");
        message.push_str(
            "- **Technology Agnostic**: Keep requirements separate from implementation\n",
        );
        message.push_str("- **Iterative Refinement**: Clarify ambiguities before planning\n");
        message.push_str("- **Structured Workflow**: Follow the phase-by-phase approach\n");
        message.push_str("- **Quality Validation**: Use checklists and analysis tools\n\n");

        message.push_str("## Additional Resources\n\n");
        message.push_str(&format!(
            "- **Agent Configuration**: {}/README.md\n",
            agent_config.folder
        ));
        message.push_str("- **Constitution Template**: .specify/templates/constitution.md\n");
        message.push_str("- **All Templates**: .specify/templates/\n\n");

        message.push_str("## Environment Verification\n\n");
        message.push_str(
            "Run `speckit_check` tool to verify your development environment and check for:\n",
        );
        message.push_str("- Required tools (git)\n");
        message.push_str("- Optional AI coding assistants (Claude, Cursor, VS Code)\n");
        message.push_str("- Overall environment readiness\n\n");

        message.push_str("---\n\n");
        message.push_str("**Status**: ✅ Ready for development\n");
        message.push_str(
            "**Next Action**: Define your project constitution or start specifying requirements\n",
        );

        Ok(message)
    }

    /// Setup agent-specific configuration
    fn setup_agent_config(&self, project_path: &Path, agent_id: &str) -> Result<()> {
        let agent_config = crate::agents::get_agent_config(agent_id)
            .ok_or_else(|| anyhow!("Unknown agent: {}", agent_id))?;

        tracing::info!(agent = %agent_id, folder = %agent_config.folder, "Setting up agent configuration");

        // Create agent-specific folder
        let agent_folder = project_path.join(&agent_config.folder);
        fs::create_dir_all(&agent_folder).with_context(|| {
            format!("Failed to create agent folder: {}", agent_folder.display())
        })?;

        // Create a basic README in the agent folder
        let readme_content = format!(
            "# {} Configuration\n\n\
            This directory contains configuration for {}.\n\n\
            For more information, visit: {}\n",
            agent_config.name, agent_config.name, agent_config.install_url
        );

        fs::write(agent_folder.join("README.md"), readme_content)
            .context("Failed to write agent README")?;

        tracing::info!("Agent configuration complete");
        Ok(())
    }

    /// Initialize git repository
    async fn init_git_repo(&self, project_path: &Path) -> Result<()> {
        tracing::info!("Initializing git repository");

        // Check if git is available
        let git_available = async_process::Command::new("git")
            .arg("--version")
            .stdout(async_process::Stdio::null())
            .stderr(async_process::Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false);

        if !git_available {
            return Err(anyhow!("git command not found"));
        }

        // Check if already a git repo
        let is_repo = async_process::Command::new("git")
            .arg("rev-parse")
            .arg("--git-dir")
            .current_dir(project_path)
            .stdout(async_process::Stdio::null())
            .stderr(async_process::Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false);

        if is_repo {
            tracing::info!("Already a git repository");
            return Ok(());
        }

        // Initialize git repo
        let status = async_process::Command::new("git")
            .arg("init")
            .current_dir(project_path)
            .stdout(async_process::Stdio::null())
            .stderr(async_process::Stdio::piped())
            .status()
            .await
            .context("Failed to execute git init")?;

        if !status.success() {
            return Err(anyhow!("git init failed with status: {}", status));
        }

        tracing::info!("Git repository initialized");
        Ok(())
    }

    /// Ensure scripts are executable (Unix only)
    #[cfg(unix)]
    fn ensure_scripts_executable(&self, project_path: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        tracing::info!("Setting execute permissions on scripts");

        let scripts_dir = project_path.join(".specify/scripts");
        if !scripts_dir.exists() {
            return Ok(());
        }

        let mut count = 0;
        for entry in walkdir::WalkDir::new(&scripts_dir) {
            let entry = entry?;
            let path = entry.path();

            // Only process .sh files
            if path.extension().and_then(|s| s.to_str()) == Some("sh") {
                // Check if it's a file (not a symlink or directory)
                if path.is_file() {
                    // Set execute permissions
                    let metadata = fs::metadata(path)?;
                    let mut permissions = metadata.permissions();
                    let mode = permissions.mode();

                    // Add execute bit for owner, group, and others if they have read permission
                    let new_mode = mode | ((mode & 0o444) >> 2);
                    permissions.set_mode(new_mode);

                    fs::set_permissions(path, permissions).with_context(|| {
                        format!("Failed to set permissions on: {}", path.display())
                    })?;

                    count += 1;
                }
            }
        }

        tracing::info!(count = count, "Set execute permissions on scripts");
        Ok(())
    }

    /// Ensure scripts are executable (Windows - no-op)
    #[cfg(not(unix))]
    #[allow(dead_code)]
    fn ensure_scripts_executable(&self, _project_path: &Path) -> Result<()> {
        // No-op on Windows
        Ok(())
    }
}

#[async_trait]
impl Tool for InitTool {
    fn definition(&self) -> ToolDefinition {
        let agent_ids = crate::agents::get_agent_ids();

        ToolDefinition {
            name: "speckit_init".to_string(),
            description: "Initialize a new spec-kit project with proper directory structure and configuration files".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_name": {
                        "type": "string",
                        "description": "Name of the project to initialize"
                    },
                    "project_path": {
                        "type": "string",
                        "description": "Path where the project should be created",
                        "default": "."
                    },
                    "ai_assistant": {
                        "type": "string",
                        "description": "AI assistant to use",
                        "enum": agent_ids,
                        "default": "claude"
                    },
                    "script_type": {
                        "type": "string",
                        "description": "Script type (sh for bash, ps for PowerShell)",
                        "enum": ["sh", "ps"],
                        "default": if cfg!(windows) { "ps" } else { "sh" }
                    },
                    "no_git": {
                        "type": "boolean",
                        "description": "Skip git repository initialization",
                        "default": false
                    }
                },
                "required": ["project_name"]
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: InitParams =
            serde_json::from_value(params).context("Failed to parse init parameters")?;

        tracing::info!(
            project_name = %params.project_name,
            project_path = %params.project_path.display(),
            ai_assistant = ?params.ai_assistant,
            script_type = ?params.script_type,
            no_git = params.no_git,
            "Initializing spec-kit project with native implementation"
        );

        // Use native Rust implementation
        match self
            .init_native(&params.project_name, &params.project_path, &params)
            .await
        {
            Ok(message) => Ok(ToolResult {
                content: vec![ContentBlock::text(message)],
                is_error: None,
            }),
            Err(e) => Ok(ToolResult {
                content: vec![ContentBlock::text(format!(
                    "Failed to initialize project: {}",
                    e
                ))],
                is_error: Some(true),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_tool_definition() {
        let tool = InitTool::new();
        let def = tool.definition();

        assert_eq!(def.name, "speckit_init");
        assert!(!def.description.is_empty());
    }

    #[tokio::test]
    async fn test_init_tool_params() {
        let params = json!({
            "project_name": "test-project",
            "project_path": "/tmp/test"
        });

        let parsed: InitParams = serde_json::from_value(params).unwrap();
        assert_eq!(parsed.project_name, "test-project");
        assert_eq!(parsed.project_path, PathBuf::from("/tmp/test"));
    }
}

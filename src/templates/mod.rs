//! Embedded spec-kit templates
//!
//! This module contains all spec-kit templates embedded directly in the binary.

/// Constitution template
pub const CONSTITUTION_TEMPLATE: &str = include_str!("../../templates/constitution.md");

/// Specification template
pub const SPEC_TEMPLATE: &str = include_str!("../../templates/spec-template.md");

/// Plan template
pub const PLAN_TEMPLATE: &str = include_str!("../../templates/plan-template.md");

/// Tasks template
pub const TASKS_TEMPLATE: &str = include_str!("../../templates/tasks-template.md");

/// Checklist template
pub const CHECKLIST_TEMPLATE: &str = include_str!("../../templates/checklist-template.md");

/// Get template by name
pub fn get_template(name: &str) -> Option<&'static str> {
    match name {
        "constitution" => Some(CONSTITUTION_TEMPLATE),
        "spec" | "specify" => Some(SPEC_TEMPLATE),
        "plan" => Some(PLAN_TEMPLATE),
        "tasks" => Some(TASKS_TEMPLATE),
        "checklist" => Some(CHECKLIST_TEMPLATE),
        _ => None,
    }
}

// Command templates (AI instructions for each tool)
pub const CONSTITUTION_COMMAND: &str = include_str!("../../templates/commands/constitution.md");
pub const SPECIFY_COMMAND: &str = include_str!("../../templates/commands/specify.md");
pub const PLAN_COMMAND: &str = include_str!("../../templates/commands/plan.md");
pub const TASKS_COMMAND: &str = include_str!("../../templates/commands/tasks.md");
pub const IMPLEMENT_COMMAND: &str = include_str!("../../templates/commands/implement.md");
pub const CLARIFY_COMMAND: &str = include_str!("../../templates/commands/clarify.md");
pub const ANALYZE_COMMAND: &str = include_str!("../../templates/commands/analyze.md");
pub const CHECKLIST_COMMAND: &str = include_str!("../../templates/commands/checklist.md");

/// Get command template (AI instructions) by name
pub fn get_command(name: &str) -> Option<&'static str> {
    match name {
        "constitution" => Some(CONSTITUTION_COMMAND),
        "specify" => Some(SPECIFY_COMMAND),
        "plan" => Some(PLAN_COMMAND),
        "tasks" => Some(TASKS_COMMAND),
        "implement" => Some(IMPLEMENT_COMMAND),
        "clarify" => Some(CLARIFY_COMMAND),
        "analyze" => Some(ANALYZE_COMMAND),
        "checklist" => Some(CHECKLIST_COMMAND),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_template() {
        assert!(get_template("constitution").is_some());
        assert!(get_template("spec").is_some());
        assert!(get_template("specify").is_some());
        assert!(get_template("plan").is_some());
        assert!(get_template("tasks").is_some());
        assert!(get_template("checklist").is_some());
        assert!(get_template("nonexistent").is_none());
    }

    #[test]
    fn test_template_content() {
        // Verify templates contain expected markers
        assert!(CONSTITUTION_TEMPLATE.contains("[PROJECT_NAME]"));
        assert!(SPEC_TEMPLATE.contains("[FEATURE NAME]"));
        assert!(PLAN_TEMPLATE.contains("[FEATURE]"));
        assert!(TASKS_TEMPLATE.contains("[FEATURE NAME]"));
        assert!(CHECKLIST_TEMPLATE.contains("[FEATURE NAME]"));
    }
}

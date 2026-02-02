# MCP Server Verification Guide

## Overview

This document verifies that all MCP tools work correctly for your local AI app.

## Features Implemented

### ✅ Core Features

1. **Native Template Download** - Downloads from GitHub without Python
2. **Agent Configuration** - Supports 17+ AI assistants (Claude, Cursor, Copilot, etc.)
3. **Script Type Selection** - Auto-detects Windows (PowerShell) vs Unix (Bash)
4. **GitHub Token Support** - Reads GH_TOKEN/GITHUB_TOKEN env vars
5. **Git Initialization** - Optional git repo setup
6. **Script Permissions** - Auto-chmod on Unix systems
7. **Rate Limit Handling** - Detects and reports GitHub API limits
8. **Path Validation** - Prevents directory traversal attacks
9. **Command Templates** - Embedded AI instructions for each tool

### ✅ All 10 MCP Tools

1. **speckit_init** - Initialize new project
2. **speckit_check** - Verify tool installation
3. **speckit_constitution** - Create project principles
4. **speckit_specify** - Define requirements
5. **speckit_plan** - Create technical plan
6. **speckit_tasks** - Generate task list
7. **speckit_implement** - Implementation guidance
8. **speckit_clarify** - Identify ambiguities
9. **speckit_analyze** - Cross-artifact analysis
10. **speckit_checklist** - Validation checklists

## MCP Server Configuration

### For Claude Desktop

Add to `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "spec-kit": {
      "command": "E:\\programming\\brand-new\\spec-kit-mcp\\target\\release\\spec-kit-mcp.exe",
      "args": [],
      "env": {}
    }
  }
}
```

### For Other MCP Clients

Use the same pattern - point to the compiled binary:

```
target/release/spec-kit-mcp.exe
```

## Tool Parameters

### 1. speckit_init

**Required**:

- `project_name` (string) - Name of the project

**Optional**:

- `project_path` (string) - Path (default: ".")
- `ai_assistant` (string) - AI to use (default: "claude")
  - Options: claude, copilot, cursor, windsurf, gemini, vscode, qwen, opencode, codex, kilocode, auggie, codebuddy, amp, shai, q, bob, qoder, kiro
- `script_type` (string) - Script type (default: auto-detect)
  - Options: sh, ps
- `no_git` (boolean) - Skip git init (default: false)
- `github_token` (string) - GitHub token (or use GH_TOKEN env var)

**Example**:

```json
{
  "project_name": "my-project",
  "ai_assistant": "claude",
  "script_type": "ps",
  "no_git": false
}
```

### 2. speckit_check

**Parameters**: None (all optional)

**Example**:

```json
{}
```

### 3. speckit_constitution

**Required**:

- `principles` (string) - Core principles

**Optional**:

- `constraints` (string) - Technical constraints
- `output_path` (string) - Output file (default: "./speckit.constitution")

**Example**:

```json
{
  "principles": "1. Simplicity First\n2. Test-Driven Development\n3. Security by Default",
  "constraints": "Must use Rust 2021 edition",
  "output_path": "./speckit.constitution"
}
```

### 4. speckit_specify

**Required**:

- `requirements` (string) - Feature requirements

**Optional**:

- `user_stories` (string) - User stories
- `output_path` (string) - Output file (default: "./speckit.specify")
- `format` (string) - Format (default: "markdown")

**Example**:

```json
{
  "requirements": "Build a user authentication system with OAuth2 support",
  "user_stories": "As a user, I want to login with Google",
  "output_path": "./speckit.specify"
}
```

### 5. speckit_plan

**Required**:

- `spec_file` (string) - Path to specification file

**Optional**:

- `tech_stack` (string) - Technology stack
- `output_path` (string) - Output file (default: "./speckit.plan")

**Example**:

```json
{
  "spec_file": "./speckit.specify",
  "tech_stack": "Rust + Tokio + PostgreSQL",
  "output_path": "./speckit.plan"
}
```

### 6. speckit_tasks

**Required**:

- `plan_file` (string) - Path to plan file

**Optional**:

- `breakdown_level` (string) - Detail level (default: "medium")
  - Options: high, medium, detailed
- `output_path` (string) - Output file (default: "./speckit.tasks")

**Example**:

```json
{
  "plan_file": "./speckit.plan",
  "breakdown_level": "detailed",
  "output_path": "./speckit.tasks"
}
```

### 7. speckit_implement

**Required**:

- `task_file` (string) - Path to tasks file

**Optional**:

- `context` (string) - Additional context
- `output_dir` (string) - Output directory (default: "./src")

**Example**:

```json
{
  "task_file": "./speckit.tasks",
  "context": "Using async/await patterns",
  "output_dir": "./src"
}
```

### 8. speckit_clarify

**Required**:

- `spec_file` (string) - Path to specification file

**Optional**:

- `questions` (array of strings) - Specific questions
- `output_path` (string) - Output file (default: "./speckit.clarify")

**Example**:

```json
{
  "spec_file": "./speckit.specify",
  "questions": ["What authentication method?", "What database?"],
  "output_path": "./speckit.clarify"
}
```

### 9. speckit_analyze

**Required**:

- `project_path` (string) - Project directory path

**Optional**:

- `check_consistency` (boolean) - Check consistency (default: true)
- `check_coverage` (boolean) - Check coverage (default: true)
- `output_path` (string) - Output file (default: "./speckit.analyze")

**Example**:

```json
{
  "project_path": ".",
  "check_consistency": true,
  "check_coverage": true,
  "output_path": "./speckit.analyze"
}
```

### 10. speckit_checklist

**Required**:

- `spec_file` (string) - Path to specification file

**Optional**:

- `include_implementation` (boolean) - Include impl items (default: true)
- `include_testing` (boolean) - Include test items (default: true)
- `output_path` (string) - Output file (default: "./speckit.checklist")

**Example**:

```json
{
  "spec_file": "./speckit.specify",
  "include_implementation": true,
  "include_testing": true,
  "output_path": "./speckit.checklist"
}
```

## Workflow Example

### Step 1: Initialize Project

```json
{
  "tool": "speckit_init",
  "params": {
    "project_name": "my-app",
    "ai_assistant": "claude",
    "script_type": "ps"
  }
}
```

### Step 2: Create Constitution

```json
{
  "tool": "speckit_constitution",
  "params": {
    "principles": "1. Simplicity\n2. Performance\n3. Security"
  }
}
```

### Step 3: Define Requirements

```json
{
  "tool": "speckit_specify",
  "params": {
    "requirements": "Build a REST API for user management"
  }
}
```

### Step 4: Create Plan

```json
{
  "tool": "speckit_plan",
  "params": {
    "spec_file": "./speckit.specify",
    "tech_stack": "Rust + Actix-web"
  }
}
```

### Step 5: Generate Tasks

```json
{
  "tool": "speckit_tasks",
  "params": {
    "plan_file": "./speckit.plan"
  }
}
```

### Step 6: Implement

```json
{
  "tool": "speckit_implement",
  "params": {
    "task_file": "./speckit.tasks"
  }
}
```

## Environment Variables

### GitHub Token (Recommended)

Set one of these to avoid rate limits:

**Windows (PowerShell)**:

```powershell
$env:GH_TOKEN = "your_github_token_here"
```

**Windows (CMD)**:

```cmd
set GH_TOKEN=your_github_token_here
```

**Unix (Bash)**:

```bash
export GH_TOKEN=your_github_token_here
```

### Logging

Control log level:

```powershell
$env:RUST_LOG = "info"  # or debug, warn, error
```

## Testing the MCP Server

### 1. Build

```powershell
cargo build --release
```

### 2. Run Directly (Test Mode)

```powershell
.\target\release\spec-kit-mcp.exe
```

### 3. Test with MCP Inspector

Install MCP Inspector:

```powershell
npm install -g @modelcontextprotocol/inspector
```

Run:

```powershell
mcp-inspector .\target\release\spec-kit-mcp.exe
```

### 4. Test with Claude Desktop

1. Add to config (see above)
2. Restart Claude Desktop
3. Look for "spec-kit" in MCP tools
4. Try: "Initialize a new spec-kit project called test-app"

## Troubleshooting

### Issue: "Rate limit exceeded"

**Solution**: Set GH_TOKEN environment variable

### Issue: "git command not found"

**Solution**: Install git or use `no_git: true`

### Issue: "Failed to download template"

**Solution**: Check internet connection, verify GitHub is accessible

### Issue: "Permission denied" (Unix)

**Solution**: Scripts are auto-chmod'd, but verify with `ls -la .specify/scripts/`

### Issue: "Agent folder not created"

**Solution**: Verify ai_assistant parameter is valid (see list above)

### Issue: "MCP server not showing in Claude"

**Solution**:

1. Check config file path
2. Verify binary path is correct
3. Restart Claude Desktop
4. Check Claude logs: `%APPDATA%\Claude\logs\`

## Verification Checklist

- [ ] Binary compiles successfully
- [ ] MCP server starts without errors
- [ ] All 10 tools are listed in MCP client
- [ ] speckit_init creates project with .specify directory
- [ ] Agent folder is created (e.g., .claude)
- [ ] Git repo is initialized (if not disabled)
- [ ] Scripts have execute permissions (Unix)
- [ ] Constitution tool creates file with template
- [ ] Specify tool creates file with template
- [ ] Plan tool creates file with template
- [ ] Tasks tool creates file with template
- [ ] All tools include command templates in output
- [ ] GitHub token is used when set
- [ ] Rate limit errors are helpful

## Success Criteria

✅ **All tools work** - No errors when calling any tool
✅ **Templates embedded** - No external file dependencies
✅ **Agent config works** - Correct folders created
✅ **Git integration works** - Repo initialized when requested
✅ **Token auth works** - Higher rate limits with token
✅ **Command templates included** - AI gets proper instructions
✅ **Cross-platform** - Works on Windows and Unix

## Next Steps

1. **Test with your AI app** - Verify all tools work in your workflow
2. **Set GitHub token** - Avoid rate limits
3. **Customize agents** - Add more agents if needed (edit `src/agents/mod.rs`)
4. **Report issues** - Document any problems found
5. **Iterate** - Improve based on real usage

## Support

For issues or questions:

1. Check logs: `$env:RUST_LOG="debug"; .\target\release\spec-kit-mcp.exe`
2. Review error messages - they include context and suggestions
3. Verify parameters match the schemas above
4. Test tools individually before full workflow

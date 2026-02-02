# Spec-Kit MCP Design

## Architecture Overview

The Spec-Kit MCP server implements a **two-phase workflow** where:

1. **MCP Tools** provide instructions and context to the AI
2. **AI Agent** follows instructions to generate fully populated content

## Design Philosophy

### Why Not Generate Content in Tools?

The MCP tools **do not** generate the final content themselves. Instead, they:

1. Validate inputs and project state
2. Return detailed instructions from `templates/commands/*.md`
3. Provide user input context
4. Specify output file path

### Why This Approach?

**Advantages:**

- **Flexibility**: AI can adapt instructions based on context
- **Intelligence**: Leverages AI's understanding and creativity
- **Maintainability**: Instructions are in markdown, easy to update
- **Consistency**: Follows the Python spec-kit design pattern

**The AI is responsible for:**

- Reading and interpreting the instructions
- Generating complete, populated content (no placeholders)
- Writing the final file to the specified path

## Tool Workflow

### Example: `speckit_constitution`

```
User calls tool with:
  - principles: "Security first, Performance matters"
  - constraints: "Must support Python 3.11+"

Tool returns:
  ## Task: Create Project Constitution

  **Output File**: ./speckit.constitution

  **Principles**:
```

Security first, Performance matters

```

**Constraints**:
```

Must support Python 3.11+

```

---

## Instructions

[Full workflow from templates/commands/constitution.md]

AI reads instructions and:
1. Parses principles and constraints
2. Generates complete constitution document
3. Writes to ./speckit.constitution
```

## Key Implementation Details

### Tool Response Format

````rust
let message = format!(
    "## Task: Create [Document Type]\n\n\
    **Output File**: {}\n\n\
    **User Input**:\n```\n{}\n```\n\n\
    ---\n\n\
    ## Instructions\n\n\
    You must follow the detailed workflow below...\n\n\
    **IMPORTANT**: Do NOT write placeholder content. \
    Generate fully populated content following the instructions.\n\n\
    {}",
    output_path,
    user_input,
    COMMAND_TEMPLATE
);
````

### Command Templates

Located in `templates/commands/*.md`:

- `constitution.md` - Constitution generation workflow
- `specify.md` - Specification generation workflow
- `plan.md` - Technical plan generation workflow
- `tasks.md` - Task breakdown workflow
- `implement.md` - Implementation workflow

These templates contain:

- Step-by-step instructions
- Validation rules
- Output format requirements
- Quality criteria

## File Structure

```
src/
├── tools/
│   ├── constitution.rs  # Returns instructions + context
│   ├── specify.rs       # Returns instructions + context
│   ├── plan.rs          # Returns instructions + context
│   ├── tasks.rs         # Returns instructions + context
│   └── implement.rs     # Returns instructions + context
├── templates/
│   └── mod.rs           # Embedded command templates
└── mcp/
    └── server.rs        # MCP protocol implementation

templates/
└── commands/
    ├── constitution.md  # AI workflow instructions
    ├── specify.md       # AI workflow instructions
    ├── plan.md          # AI workflow instructions
    ├── tasks.md         # AI workflow instructions
    └── implement.md     # AI workflow instructions
```

## Comparison with Python Implementation

| Aspect               | Python Spec-Kit                       | Rust MCP                                 |
| -------------------- | ------------------------------------- | ---------------------------------------- |
| **Phase 1**          | Download ZIP, extract templates       | Return instructions in tool response     |
| **Phase 2**          | AI reads command files from disk      | AI reads instructions from tool response |
| **Template Storage** | Files in `.specify/templates/`        | Embedded in binary                       |
| **AI Instructions**  | Separate `.md` files                  | Embedded in tool responses               |
| **File Creation**    | AI creates after reading instructions | AI creates after receiving instructions  |

## Usage Example

```rust
// User calls MCP tool
speckit_constitution(
    principles: "Security first, Test-driven development",
    constraints: "PostgreSQL only, No external APIs"
)

// Tool returns instructions (not file)
// AI (Kiro) receives instructions
// AI generates complete constitution
// AI writes to ./speckit.constitution

// Result: Fully populated file with NO placeholders
```

## Critical Rules for AI Agents

When using these tools:

1. **Read the full instructions** returned by the tool
2. **Follow the workflow** step-by-step
3. **Generate complete content** - no placeholders like `[TODO]` or `[FILL IN]`
4. **Write the file** to the specified output path
5. **Validate** the content meets the quality criteria

## Benefits

- **Separation of Concerns**: Tools handle validation, AI handles generation
- **Flexibility**: AI can adapt to context and user preferences
- **Maintainability**: Instructions are markdown, not code
- **Consistency**: Matches the proven Python spec-kit design
- **Intelligence**: Leverages AI capabilities fully

## Future Enhancements

Potential improvements:

- Add validation hooks after AI generates content
- Support for custom instruction templates
- Progress tracking for multi-step workflows
- Integration with version control systems

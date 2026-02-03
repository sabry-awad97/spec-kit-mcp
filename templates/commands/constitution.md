---
description: Create or update the project constitution from interactive or provided principle inputs, ensuring all dependent templates stay in sync.
handoffs:
  - label: Build Specification
    agent: speckit.specify
    prompt: Implement the feature specification based on the updated constitution. I want to build...
---

## Role & Expertise

You are a **Senior Engineering Manager** with 15+ years of experience in establishing engineering standards and governance. Your expertise includes:

- Defining clear, enforceable engineering principles
- Balancing flexibility with consistency
- Creating governance frameworks that scale
- Aligning technical decisions with business goals
- Managing technical debt and quality standards

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Structured Thinking Protocol

Before updating the constitution, complete these analysis steps:

### [UNDERSTAND]

- Review current constitution (if exists)
- Identify principles being added, modified, or removed
- Extract governance requirements from user input
- Note any conflicts with existing principles

### [ANALYZE]

- Assess impact of changes on existing projects
- Identify dependencies on other templates and docs
- Recognize versioning implications (major/minor/patch)
- Evaluate enforcement mechanisms needed

### [STRATEGIZE]

- Plan version bump based on change type
- Determine which templates need updates
- Prepare consistency propagation checklist
- Design validation approach

### [EXECUTE]

- Update constitution with concrete values
- Propagate changes to dependent artifacts
- Generate sync impact report
- Validate consistency across all files

## Goal

You are updating the project constitution at `/memory/constitution.md`. This file is a TEMPLATE containing placeholder tokens in square brackets (e.g. `[PROJECT_NAME]`, `[PRINCIPLE_1_NAME]`). Your job is to (a) collect/derive concrete values, (b) fill template precisely, and (c) propagate any amendments across dependent artifacts.

## Execution Flow

### 1. Load Existing Constitution

Load existing constitution template at `/memory/constitution.md`.

**Identify every placeholder token** of form `[ALL_CAPS_IDENTIFIER]`.

**IMPORTANT**: User might require less or more principles than ones used in template. If number is specified, respect that - follow general template. You will update doc accordingly.

### 2. Collect/Derive Values for Placeholders

**For each placeholder**:

- If user input (conversation) supplies value, use it
- Otherwise infer from existing repo context (README, docs, prior constitution versions if embedded)

**For governance dates**:

- `RATIFICATION_DATE`: Original adoption date (if unknown ask or mark TODO)
- `LAST_AMENDED_DATE`: Today if changes are made, otherwise keep previous

**For `CONSTITUTION_VERSION`**: Must increment according to semantic versioning rules:

- **MAJOR**: Backward incompatible governance/principle removals or redefinitions
- **MINOR**: New principle/section added or materially expanded guidance
- **PATCH**: Clarifications, wording, typo fixes, non-semantic refinements

If version bump type ambiguous, propose reasoning before finalizing.

### 3. Draft Updated Constitution Content

**Replace every placeholder** with concrete text (no bracketed tokens left except intentionally retained template slots that project has chosen not to define yet—explicitly justify any left).

**Preserve heading hierarchy** and comments can be removed once replaced unless they still add clarifying guidance.

**Ensure each Principle section**:

- Succinct name line
- Paragraph (or bullet list) capturing non-negotiable rules
- Explicit rationale if not obvious

**Ensure Governance section** lists:

- Amendment procedure
- Versioning policy
- Compliance review expectations

### 4. Consistency Propagation Checklist

Convert prior checklist into active validations:

**Read and update if needed**:

- `/templates/plan-template.md`: Ensure any "Constitution Check" or rules align with updated principles
- `/templates/spec-template.md`: For scope/requirements alignment—update if constitution adds/removes mandatory sections or constraints
- `/templates/tasks-template.md`: Ensure task categorization reflects new or removed principle-driven task types (e.g., observability, versioning, testing discipline)
- Each command file in `/templates/commands/*.md`: Verify no outdated references remain when generic guidance is required
- Runtime guidance docs (e.g., `README.md`, `docs/quickstart.md`, or agent-specific guidance files if present): Update references to principles changed

### 5. Produce Sync Impact Report

Prepend as HTML comment at top of constitution file after update:

```html
<!--
SYNC IMPACT REPORT
Version Change: vX.Y.Z → vA.B.C
Modified Principles:
- [Old Title] → [New Title] (if renamed)
Added Sections:
- [Section Name]
Removed Sections:
- [Section Name]
Templates Requiring Updates:
- ✅ /templates/plan-template.md (updated)
- ⚠️ /templates/spec-template.md (pending)
Follow-up TODOs:
- [Any placeholders intentionally deferred]
-->
```

### 6. Validation Before Final Output

**Check**:

- No remaining unexplained bracket tokens
- Version line matches report
- Dates ISO format YYYY-MM-DD
- Principles are declarative, testable, free of vague language ("should" → replace with MUST/SHOULD rationale where appropriate)

### 7. Write Completed Constitution

Write completed constitution back to `/memory/constitution.md` (overwrite).

### 8. Output Final Summary

**Report to user**:

- New version and bump rationale
- Any files flagged for manual follow-up
- Suggested commit message (e.g., `docs: amend constitution to vX.Y.Z (principle additions + governance update)`)

## Chain-of-Verification (Self-Check)

After updating constitution, perform this verification:

### Step 1: Generate Verification Questions

Create 5 questions that would expose errors in your constitution update:

1. "Are all principles clear, testable, and enforceable?"
2. "Is the version bump appropriate for the type of changes made?"
3. "Have all dependent templates been identified and updated?"
4. "Are there any contradictions between principles?"
5. "Can developers easily understand what's required vs. recommended?"

### Step 2: Answer Each Question

Review your constitution update against each question and note any issues found.

### Step 3: Provide Confidence Assessment

**Confidence Level**: [0-100%]

**Key Assumptions**:

- [List assumptions about project context]

**What Would Change This Constitution**:

- [Factors that would require revision]

**Alternative Principles** (if confidence <80%):

- [Describe alternative governance approach]

## Formatting & Style Requirements

- Use Markdown headings exactly as in template (do not demote/promote levels)
- Wrap long rationale lines to keep readability (<100 chars ideally) but do not hard enforce with awkward breaks
- Keep single blank line between sections
- Avoid trailing whitespace

**If user supplies partial updates** (e.g., only one principle revision), still perform validation and version decision steps.

**If critical info missing** (e.g., ratification date truly unknown), insert `TODO(<FIELD_NAME>): explanation` and include in Sync Impact Report under deferred items.

**Do not create new template**; always operate on existing `/memory/constitution.md` file.

## Negative Examples (What NOT To Do)

### ❌ BAD: Vague Principles

```markdown
## Principle: Code Quality

We should write good code that is maintainable and follows best practices.
```

**Why it's bad**: Subjective terms ("good", "maintainable"), no measurable criteria, not enforceable

### ✅ GOOD: Clear, Testable Principles

```markdown
## Principle: Code Quality Standards

**Rule**: All production code MUST:

- Pass automated linting (ESLint/Prettier with project config)
- Achieve minimum 80% test coverage for business logic
- Include JSDoc comments for all public APIs
- Pass code review by at least one other engineer

**Rationale**: Consistent code quality reduces bugs, improves maintainability, and enables team scalability.

**Enforcement**: CI pipeline blocks merges that fail linting or coverage thresholds.
```

**Why it's good**: Specific, measurable, enforceable, clear rationale

### ❌ BAD: Contradictory Principles

```markdown
## Principle: Move Fast

Ship features quickly to meet market demands.

## Principle: Zero Bugs

Never ship code with known bugs.
```

**Why it's bad**: Principles contradict each other, no guidance on trade-offs

### ✅ GOOD: Balanced Principles with Trade-offs

```markdown
## Principle: Balanced Velocity

**Rule**: Ship features incrementally with acceptable quality bar:

- P0 bugs MUST be fixed before release
- P1 bugs MAY ship with documented workarounds and fix timeline
- P2+ bugs tracked but don't block release

**Rationale**: Balances speed-to-market with quality. Allows learning from real usage while maintaining user trust.

**Trade-off**: Accept some technical debt in exchange for faster feedback loops.
```

**Why it's good**: Acknowledges trade-offs, provides clear decision framework, realistic

## Final Output Format

Your final response must include:

1. **Constitution Version**: [old] → [new]
2. **Version Bump Type**: Major/Minor/Patch with rationale
3. **Principles Modified**: [list]
4. **Principles Added**: [list]
5. **Principles Removed**: [list]
6. **Templates Updated**: [list with status]
7. **Confidence Level**: [0-100%]
8. **Follow-up Actions**: [list if any]
9. **Suggested Commit Message**: [message]

# 🧪 Complete MCP Test Scenario

**Test all 10 Spec-Kit MCP tools in a realistic workflow**

---

## 📋 Test Project: Todo CLI Application

We'll build a simple todo CLI app to test all tools.

---

## 🎯 Step-by-Step Test Instructions

### Step 1: Check Environment ✅

**Tool:** `speckit_check`

**Command:**

```
Use speckit_check to verify my development environment
```

**Expected Output:**

```
✅ uv/uvx is available
✅ Spec-kit can be run via: uvx --from git+https://github.com/github/spec-kit.git specify
✅ git is available
✅ All required tools are installed!

You're ready to use spec-kit for spec-driven development.
```

**What to verify:**

- ✅ No errors
- ✅ All tools detected
- ✅ Ready message displayed

---

### Step 2: Initialize Project 🎬

**Tool:** `speckit_init`

**Command:**

```
Use speckit_init with:
- project_name: "test-project"
- project_path: "./test-project"
```

**Expected Output:**

```
Successfully initialized spec-kit project 'test-project' at ./test-project

Next steps:
1. Navigate to the project: cd ./test-project
2. Create constitution: Use speckit_constitution tool
3. Define requirements: Use speckit_specify tool
```

**What to verify:**

- ✅ `.specify/` directory created
- ✅ Templates directory exists
- ✅ Scripts directory exists
- ✅ Memory directory exists

**Check files:**

```powershell
ls test-project/.specify
```

---

### Step 3: Create Constitution 📜

**Tool:** `speckit_constitution`

**Command:**

```
Use speckit_constitution with:
- principles: "Simplicity: Keep code simple and readable. CLI-first: Focus on command-line interface. No dependencies: Use only Python standard library. User-friendly: Clear error messages and help text."
- constraints: "Must work on Python 3.11+. Single file implementation. No database required. Cross-platform compatible."
- output_path: "./test-project/speckit.constitution"
```

**Expected Output:**

```
Constitution created successfully at ./test-project/speckit.constitution

The constitution defines:
- Core principles that guide development
- Technical constraints and boundaries
- Standards for code quality and architecture

Next step: Use speckit_specify tool to define requirements
```

**What to verify:**

- ✅ File created at `test-project/speckit.constitution`
- ✅ Contains principles section
- ✅ Contains constraints section

**Check content:**

```powershell
cat test-project/speckit.constitution
```

---

### Step 4: Define Requirements 📝

**Tool:** `speckit_specify`

**Command:**

```
Use speckit_specify with:
- requirements: "Build a command-line todo application with the following features: Add tasks with descriptions, List all tasks with their status (pending/completed), Mark tasks as complete by ID, Delete tasks by ID, Persist tasks to a JSON file (tasks.json), Display task count and completion statistics, Provide clear help text for all commands"
- user_stories: "As a user, I want to add tasks so I can track my work. As a user, I want to list all tasks so I can see what needs to be done. As a user, I want to mark tasks as complete so I can track my progress. As a user, I want to delete tasks so I can remove completed or unwanted items. As a user, I want my tasks to persist between sessions so I don't lose my data."
- output_path: "./test-project/speckit.specify"
- format: "markdown"
```

**Expected Output:**

```
Specification created successfully at ./test-project/speckit.specify

The specification defines:
- What needs to be built (requirements)
- Who it's for and why (user stories)
- Success criteria (acceptance criteria)

Next step: Use speckit_plan tool to create a technical plan
```

**What to verify:**

- ✅ File created at `test-project/speckit.specify`
- ✅ Contains requirements section
- ✅ Contains user stories section

**Check content:**

```powershell
cat test-project/speckit.specify
```

---

### Step 5: Clarify Requirements ❓

**Tool:** `speckit_clarify`

**Command:**

```
Use speckit_clarify with:
- spec_file: "./test-project/speckit.specify"
- output_path: "./test-project/speckit.clarify"
```

**Expected Output:**

```
Clarification analysis complete!

Analyzed: ./test-project/speckit.specify
Issues found: 3
Output: ./test-project/speckit.clarify

⚠ Please review and address the identified issues
```

**What to verify:**

- ✅ File created at `test-project/speckit.clarify`
- ✅ Lists ambiguities found
- ✅ Provides recommendations

**Check content:**

```powershell
cat test-project/speckit.clarify
```

**Expected issues:**

- Performance requirements not specified
- Error handling approach not specified
- Testing strategy not specified

---

### Step 6: Create Technical Plan 🗺️

**Tool:** `speckit_plan`

**Command:**

```
Use speckit_plan with:
- spec_file: "./test-project/speckit.specify"
- tech_stack: "Python 3.11+ with argparse for CLI parsing, JSON module for data persistence, dataclasses for Task model, pathlib for file operations"
- output_path: "./test-project/speckit.plan"
```

**Expected Output:**

```
Technical plan created successfully at ./test-project/speckit.plan

The plan includes:
- Architecture and system design
- Technology stack and frameworks
- Implementation approach
- Module breakdown

Next step: Use speckit_tasks tool to generate actionable tasks
```

**What to verify:**

- ✅ File created at `test-project/speckit.plan`
- ✅ Contains architecture section
- ✅ Contains tech stack section
- ✅ Contains implementation approach
- ✅ References the specification

**Check content:**

```powershell
cat test-project/speckit.plan
```

---

### Step 7: Analyze Consistency 🔍

**Tool:** `speckit_analyze`

**Command:**

```
Use speckit_analyze with:
- project_path: "./test-project"
- check_consistency: true
- check_coverage: true
- output_path: "./test-project/speckit.analyze"
```

**Expected Output:**

```
Analysis complete!

Project: ./test-project
Consistency: Checked
Coverage: Checked
Output: ./test-project/speckit.analyze

Review the analysis report for any issues or recommendations.
```

**What to verify:**

- ✅ File created at `test-project/speckit.analyze`
- ✅ Checks constitution alignment
- ✅ Checks requirement coverage
- ✅ Provides recommendations

**Check content:**

```powershell
cat test-project/speckit.analyze
```

---

### Step 8: Generate Task List ✅

**Tool:** `speckit_tasks`

**Command:**

```
Use speckit_tasks with:
- plan_file: "./test-project/speckit.plan"
- breakdown_level: "medium"
- output_path: "./test-project/speckit.tasks"
```

**Expected Output:**

```
Task list generated successfully at ./test-project/speckit.tasks

The task list includes:
- Prioritized actionable items
- Clear acceptance criteria
- Dependencies between tasks
- Estimated effort levels

Next step: Use speckit_implement tool to execute the tasks
```

**What to verify:**

- ✅ File created at `test-project/speckit.tasks`
- ✅ Contains task breakdown
- ✅ Tasks have acceptance criteria
- ✅ Tasks have dependencies
- ✅ References the plan

**Check content:**

```powershell
cat test-project/speckit.tasks
```

**Expected tasks:**

- Create Task dataclass
- Implement TaskManager class
- Add CLI argument parser
- Implement add command
- Implement list command
- Implement complete command
- Implement delete command
- Add JSON persistence
- Add error handling
- Add help text

---

### Step 9: Generate Checklist 📋

**Tool:** `speckit_checklist`

**Command:**

```
Use speckit_checklist with:
- spec_file: "./test-project/speckit.specify"
- include_implementation: true
- include_testing: true
- output_path: "./test-project/speckit.checklist"
```

**Expected Output:**

```
Checklist generated successfully at ./test-project/speckit.checklist

The checklist includes:
- Requirements validation items
- Implementation validation items
- Testing validation items

Use this checklist to validate your implementation.
```

**What to verify:**

- ✅ File created at `test-project/speckit.checklist`
- ✅ Contains requirements checklist
- ✅ Contains implementation checklist
- ✅ Contains testing checklist

**Check content:**

```powershell
cat test-project/speckit.checklist
```

---

### Step 10: Implement 💻

**Tool:** `speckit_implement`

**Command:**

```
Use speckit_implement with:
- task_file: "./test-project/speckit.tasks"
- context: "Follow PEP 8 style guide. Add comprehensive docstrings. Use type hints. Include error handling for all file operations. Add helpful error messages."
- output_dir: "./test-project"
```

**Expected Output:**

```
Implementation guidance provided!

Task file: ./test-project/speckit.tasks
Output directory: ./test-project
Context: Follow PEP 8 style guide. Add comprehensive docstrings...

The AI will now implement each task according to the plan and constitution.
```

**What to verify:**

- ✅ Implementation files created
- ✅ Code follows constitution principles
- ✅ All tasks implemented
- ✅ Error handling included
- ✅ Documentation added

**Check generated files:**

```powershell
ls test-project/*.py
```

---

## 📊 Verification Checklist

After running all tools, verify the following files exist:

```
test-project/
├── .specify/                    # ← Created by speckit_init
│   ├── memory/
│   ├── scripts/
│   ├── specs/
│   └── templates/
├── speckit.constitution         # ← Created by speckit_constitution
├── speckit.specify              # ← Created by speckit_specify
├── speckit.clarify              # ← Created by speckit_clarify
├── speckit.plan                 # ← Created by speckit_plan
├── speckit.analyze              # ← Created by speckit_analyze
├── speckit.tasks                # ← Created by speckit_tasks
├── speckit.checklist            # ← Created by speckit_checklist
└── todo.py                      # ← Created by speckit_implement
```

---

## 🧪 Testing the Generated Application

After implementation, test the todo app:

```powershell
# Navigate to test project
cd test-project

# Test add command
python todo.py add "Test the MCP server"
python todo.py add "Write documentation"
python todo.py add "Deploy to production"

# Test list command
python todo.py list

# Test complete command
python todo.py complete 1

# Test list again to see completion
python todo.py list

# Test delete command
python todo.py delete 2

# Test final list
python todo.py list

# Test help
python todo.py --help
```

**Expected behavior:**

- ✅ Tasks are added successfully
- ✅ Tasks are listed with status
- ✅ Tasks can be marked complete
- ✅ Tasks can be deleted
- ✅ Data persists to tasks.json
- ✅ Help text is clear and useful

---

## 📈 Success Criteria

### All Tools Tested ✅

- [x] speckit_check - Environment verified
- [x] speckit_init - Project initialized
- [x] speckit_constitution - Principles defined
- [x] speckit_specify - Requirements documented
- [x] speckit_clarify - Ambiguities identified
- [x] speckit_plan - Technical plan created
- [x] speckit_analyze - Consistency checked
- [x] speckit_tasks - Tasks generated
- [x] speckit_checklist - Validation checklist created
- [x] speckit_implement - Code generated

### Workflow Validated ✅

- [x] Tools work in sequence
- [x] Each tool produces expected output
- [x] Files are created in correct locations
- [x] Content follows expected format
- [x] Generated code is functional

### Quality Verified ✅

- [x] Constitution guides implementation
- [x] All requirements covered
- [x] Code follows best practices
- [x] Error handling included
- [x] Documentation present

---

## 🎉 Test Complete!

If all steps pass, the MCP server is working correctly and ready for production use!

---

## 🐛 Troubleshooting

### Issue: Tool not found

**Solution:** Ensure MCP server is configured in your AI assistant's config file.

### Issue: .specify directory not found

**Solution:** Run `speckit_init` first before other tools.

### Issue: File not found errors

**Solution:** Check that previous steps completed successfully and files were created.

### Issue: uvx takes long on first run

**Expected:** First run downloads spec-kit (~5-10 seconds). Subsequent runs are fast.

---

## 📝 Notes

- This test scenario covers the complete spec-driven development workflow
- Each tool builds on the previous one
- The workflow demonstrates real-world usage
- All 10 tools are tested in a logical sequence
- The final output is a working application

---

**Ready to test? Start with Step 1!** 🚀

Create a new task file in `tasks/`. Before writing, study the codebase to deeply understand the problem and the affected code paths.

## Process

1. **Determine the next task number.** List files in `tasks/` and pick the next sequential number (e.g., if `11-ux-polish.md` exists, the new file is `12-something.md`).

2. **Understand the problem.** Read all source files relevant to the user's request. Trace the full code path — frontend components, IPC commands, core logic, database layer. Do NOT write the task until you understand the current behavior thoroughly.

3. **Write the task file** at `tasks/NN-short-slug.md` following the structure below.

## Task File Structure

Every task file MUST contain these sections in order:

### Header
```markdown
# Task NN: Short Descriptive Title

**Track:** <area> — <type> (e.g., "B — Classification Pipeline (frontend + backend)")
**Blocked by:** <task numbers or "nothing">
**Blocks:** <task numbers or "nothing">
```

### Problem
Explain what's wrong or missing. Use concrete examples from the actual codebase — real data, real file paths, real line numbers. Show why the current behavior is broken or insufficient. No abstract hand-waving.

### Current State
Document the relevant code as it exists today. For each affected file:
- File path and line numbers
- What the code does now
- Key structs, functions, IPC commands involved
- Include short code snippets where they clarify the situation

This section should give someone enough context to start implementing without having to read the whole codebase first.

### Scope
Numbered sub-tasks, each describing one discrete change:
- **Which file** to change
- **What to change** — specific enough to implement from, with code sketches where helpful
- **How it connects** to other changes in the task

Each sub-task should be independently understandable. Prefer small, focused changes over monolithic rewrites.

### Files to Change
A table summarizing all affected files:
```markdown
| File | Change |
|---|---|
| `path/to/file` | Brief description of what changes |
```

### Test Scenarios
Concrete test cases, split into:
- **Backend (Rust unit tests)** — if applicable
- **Frontend (manual UI tests)** — numbered, specific, verifiable

Each test should describe an action and expected outcome, not vague "test that it works."

### Acceptance Criteria
Bulleted list of conditions that must ALL be true for the task to be considered done. These are the "definition of done" — specific, measurable, no ambiguity.

## Rules

- **Be specific.** Every claim must reference actual files, line numbers, function names. No generic descriptions.
- **Show real examples.** Use data from the actual app or realistic test data, not abstract placeholders.
- **Code sketches, not implementations.** Show enough code to communicate intent and approach, but don't write the full implementation — that's for the implementer.
- **No unnecessary scope.** Only include changes that directly solve the stated problem. Don't bundle in refactors, cleanups, or nice-to-haves.
- **Preserve backward compatibility** unless breaking it is the explicit point of the task.
- **No new DB tables or migrations** unless genuinely required — prefer extending existing structures.

## After writing

Report the file path and a 2-3 sentence summary of the task to the user.

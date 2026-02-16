Perform a comprehensive audit of this codebase. Read all source files, tests, and docs before reporting. Do NOT fix anything — only report findings.

Run all checks in parallel where possible using subagents. Output a single prioritized report at the end.

## 1. Rust Code Quality

Read all `.rs` files in `crates/core/src/` and `src-tauri/src/`.

Check for:
- `.unwrap()` / `.expect()` in non-test code (should use proper error handling)
- Unnecessary `.clone()` where references would work
- Dead code: unused functions, structs, enum variants, trait impls
- `pub` visibility that should be `pub(crate)` or private
- Missing `#[must_use]` on functions that return Results
- Mutex held across await points or longer than necessary
- Raw string matching where enums or typed parsing would be safer
- Panicking indexing (`[i]`) where `.get(i)` would be safer

## 2. Svelte & Frontend

Read all `.svelte` files in `src/` and `src/lib/`.

Check for:
- **Svelte 4 regressions**: `export let`, `$:`, `on:click`, `createEventDispatcher`, `new App()` — these are WRONG, must use Svelte 5 syntax
- Hardcoded values that should be constants or props
- Missing error handling on `invoke()` calls (uncaught promise rejections)
- Reactive state that should be `$state()` but isn't
- Components doing too many things (>150 lines of logic = flag it)
- Accessibility: missing aria labels, non-semantic HTML, click handlers on non-interactive elements
- Inline styles that should be Tailwind classes

## 3. Database Patterns

Read `crates/core/src/db.rs` thoroughly.

Check for:
- Missing indices on columns used in WHERE/ORDER BY
- N+1 query patterns (looping with individual queries instead of batch)
- Transactions missing where multiple writes should be atomic
- SQL injection risk (string interpolation instead of parameterized queries)
- Schema issues: missing NOT NULL constraints, missing DEFAULT values, TEXT where INTEGER would be better
- Connection handling: is the single Mutex<Database> a bottleneck? Could it deadlock?
- Missing UNIQUE constraints that the app logic assumes

## 4. Test Coverage

Read all `#[cfg(test)]` modules and compare against the public API surface.

Check for:
- Public functions/methods with NO test coverage
- Edge cases not tested: empty input, malformed data, Unicode, very large inputs
- Tests that only test the happy path
- Missing integration tests (e.g., parse → classify → save roundtrip)
- Test helpers or fixtures that could reduce duplication

## 5. Overengineering

Check for:
- Traits with only one implementation and no clear plan for more
- Abstractions that add indirection without value
- Generic type parameters that are always the same concrete type
- Builder patterns or config structs for simple operations
- Feature flags, compatibility shims, or dead conditional branches
- Modules with more boilerplate than logic

## 6. Docs Compatibility

Read `CLAUDE.md` and `README.md` (if exists). Compare against actual code.

Check for:
- Commands listed in docs that don't work
- Architecture descriptions that don't match the actual code
- Documented features that aren't implemented
- Implemented features not mentioned in docs
- Outdated dependency versions mentioned in docs

## Output Format

Produce a single report grouped by severity:

### 🔴 Critical
Issues that will cause bugs, data loss, or security problems.

### 🟡 Warning
Code smells, missing coverage, or patterns that will cause pain as the codebase grows.

### 🔵 Suggestion
Minor improvements, style consistency, or nice-to-haves.

Each finding must include:
- **File and line reference** (e.g., `crates/core/src/db.rs:142`)
- **What's wrong** (one sentence)
- **Why it matters** (one sentence)

Do NOT include generic advice. Every finding must reference specific code.

# Task 92: CI Pipeline & GitHub Issue Templates

## Goal

Add a CI workflow so PRs get automated feedback, and issue templates so contributors can file structured reports.

## Deliverables

### 1. CI workflow: `.github/workflows/ci.yml`

Triggers: `push` to `main`, `pull_request` to `main`.

Jobs:

**test (matrix: ubuntu-latest, macos-latest, windows-latest):**
- Checkout
- Install Rust stable
- Install Node 18
- `npm install`
- Install Tauri system dependencies (Linux: `libwebkit2gtk-4.1-dev` etc.)
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `npm run build` (frontend only — verifies Svelte/Tailwind compilation)

Notes:
- Use `actions/cache` for Cargo registry + target dir + node_modules
- The existing `release.yml` handles builds on tag push — this workflow is for PR validation only
- `cargo clippy` should fail on warnings to enforce lint quality
- Windows/Linux builds may need Tauri system deps — check Tauri docs for CI setup

### 2. Issue templates

**`.github/ISSUE_TEMPLATE/bug_report.md`:**
```markdown
---
name: Bug Report
about: Report a bug or unexpected behavior
title: ''
labels: bug
assignees: ''
---

**Describe the bug**
A clear description of the problem.

**To reproduce**
Steps to reproduce:
1. Go to '...'
2. Click on '...'
3. See error

**Expected behavior**
What you expected to happen.

**Screenshots**
If applicable.

**Environment:**
- OS: [e.g. macOS 15.2, Windows 11, Ubuntu 24.04]
- App version: [e.g. 0.1.0]

**Additional context**
Any other details.
```

**`.github/ISSUE_TEMPLATE/feature_request.md`:**
```markdown
---
name: Feature Request
about: Suggest a new feature or improvement
title: ''
labels: enhancement
assignees: ''
---

**Problem**
What problem does this solve?

**Proposed solution**
How should it work?

**Alternatives considered**
Other approaches you've thought about.

**Additional context**
Mockups, examples, or related issues.
```

## Files to create
- `.github/workflows/ci.yml`
- `.github/ISSUE_TEMPLATE/bug_report.md`
- `.github/ISSUE_TEMPLATE/feature_request.md`

## Notes
- The release workflow (task 83) already exists at `.github/workflows/release.yml` — don't duplicate or conflict
- Tauri CI setup reference: https://v2.tauri.app/distribute/ci-cd/
- Consider whether Windows CI is worth the build time — can start with Linux + macOS only

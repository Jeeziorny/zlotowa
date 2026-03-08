# Task 102: CI/CD Pipeline

## Goal

Set up a CI/CD pipeline for the monorepo (Rust + Svelte/JS). Run tests, linting, and formatting checks on PRs and pushes to main.

## Big Picture

This is a Tauri v2 desktop app with a Cargo workspace (3 crates) and a Svelte 5 + Tailwind CSS v4 frontend. CI should catch regressions in both languages before merge.

**Rough coverage areas:**
- Rust: `cargo test`, `cargo clippy`, `cargo fmt --check`
- Frontend: `npm run build` (Svelte/Tailwind compilation), consider ESLint or similar
- Cross-platform: at minimum Linux + macOS (the two release targets), Windows optional

**Open questions to resolve during refinement:**
- Which CI provider? (GitHub Actions most likely — `.github/workflows/release.yml` already exists)
- Does task 92's `ci.yml` already exist or was it never merged? If it exists, this task extends it; if not, this replaces it.
- Matrix strategy: which OS/Rust/Node version combos?
- Caching strategy for Cargo registry, target dir, node_modules
- Whether to add ESLint/Prettier for JS/Svelte (and which config)
- Whether Tauri system deps are needed for `cargo test` or only for `cargo build`
- Run time budget — how long is acceptable for a PR check?

## Notes
- Exact CI actions, job structure, and tool choices to be outlined during refinement — this ticket captures intent only.
- Existing reference: `tasks/92-ci-and-issue-templates.md` (may or may not be deployed).
- Existing release workflow: `.github/workflows/release.yml` (tag-triggered builds).

# Task 91: Open Source Files & Package Metadata

## Goal

Add all standard open-source project files and update package metadata so the repo is ready for public release.

## Deliverables

### 1. LICENSE (MIT)

Create `LICENSE` in project root with MIT license text, copyright holder "Kamil" (or full name — confirm with user), year 2024-present.

### 2. CONTRIBUTING.md

Create `CONTRIBUTING.md` covering:
- Prerequisites (Rust stable, Node 18+, Tauri v2 system deps)
- Dev setup (`git clone`, `npm install`, `npm run tauri dev`)
- Running tests (`cargo test`, `cargo test -p accountant-core`)
- Project structure overview (Cargo workspace: `crates/core`, `crates/cli`, `src-tauri`; frontend in `src/`)
- Coding conventions:
  - Svelte 5 syntax (`$state()`, `$derived()`, `$props()`, `onclick={}`)
  - Tailwind v4
  - Dark theme palette (gray-950/900/800, emerald accents)
  - Rust errors via `thiserror`, Tauri commands return `Result<T, String>`
- PR process: fork, branch, test, submit PR
- Link to Code of Conduct

### 3. CODE_OF_CONDUCT.md

Adopt Contributor Covenant v2.1 (standard text). Set enforcement contact email (confirm with user).

### 4. SECURITY.md

Create `SECURITY.md` covering:
- Supported versions (currently only latest)
- How to report vulnerabilities (email, not public issue)
- Scope: SQLite database with financial data, LLM API key storage in config table, CSV parsing
- Response timeline expectations

### 5. Package metadata updates

**Cargo.toml (workspace root):**
```toml
[workspace.package]
license = "MIT"
```

**crates/core/Cargo.toml** — add:
```toml
license.workspace = true
```

**crates/cli/Cargo.toml** — add:
```toml
license.workspace = true
```

**src-tauri/Cargo.toml** — add:
```toml
license.workspace = true
```

**package.json** — update:
- Remove `"private": true`
- Add `"license": "MIT"`
- Add `"description": "Desktop expense tracking and classification app"`
- Add `"repository"` field (confirm URL with user)
- Add `"homepage"` field (point to GitHub repo or docs)

## Files to create
- `LICENSE`
- `CONTRIBUTING.md`
- `CODE_OF_CONDUCT.md`
- `SECURITY.md`

## Files to modify
- `Cargo.toml` (workspace root)
- `crates/core/Cargo.toml`
- `crates/cli/Cargo.toml`
- `src-tauri/Cargo.toml`
- `package.json`

## Notes
- Confirm copyright holder name and contact email before creating LICENSE and SECURITY.md
- The `"private": true` removal means npm won't block `npm publish`, but since this is a Tauri app (not an npm package), that's fine

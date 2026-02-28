# Task 69 — Rebrand to "złotówa" & Sidebar Branding

Merges items: #1 (visual branding), #3 (version label context), plus full app rebrand.

## Problem

The app is called "4ccountant" everywhere — sidebar, version footer, window title, Tauri config, Cargo manifests, docs. We're rebranding to **złotówa** (Polish slang for a miser, from "złoty").

The sidebar header has no visual identity. The footer shows a bare `v0.1.0` with no context.

## Solution

### 1. Full Rebrand — "4ccountant" → "złotówa"

Replace every occurrence of "4ccountant" with "złotówa" across the entire codebase:

- **Tauri config** (`src-tauri/tauri.conf.json`) — app name, window title
- **Cargo manifests** (`src-tauri/Cargo.toml`, `crates/*/Cargo.toml`, root `Cargo.toml`) — package names, descriptions
- **Rust source** — any user-facing strings referencing the app name (DB path, CLI output, etc.)
- **Frontend** — sidebar wordmark, page titles, any UI text
- **Docs** — `docs/`, `CLAUDE.md`, `README.md` if present
- **Task board** — `task-board.md` header if applicable

Note: internal crate names (e.g. `accountant-core`) can stay as-is or be updated — decide during implementation.

### 2. Logo Mark (Sidebar)

Use the pixel-art gold coin with "ZŁO" text (`icon-zloo.svg`) — embed as inline SVG in the sidebar header, to the left of the wordmark. Pick an appropriate size (w-7 to w-10). The header remains a clickable button that navigates to dashboard.

### 3. App Icon (Dock / App Switcher)

Generate macOS/Windows app icons from `icon-zloo.svg` to replace the default Tauri green square:

- Scale up using nearest-neighbor interpolation (preserves pixel-art crispness)
- Generate required sizes: `32x32.png`, `128x128.png`, `128x128@2x.png` (256x256)
- Build `icon.icns` (macOS) and `icon.ico` (Windows) from the PNGs
- Place in `src-tauri/icons/`, overwriting the defaults

### 4. Version Footer

Replace the bare version string with a branded footer line:
```
złotówa v0.1.0
```

Add `border-t border-gray-800/50 pt-3` separator above it. Keep `text-xs text-gray-600`.

## Files

| File | Action |
|------|--------|
| `src/lib/Sidebar.svelte` | Modify — logo SVG, wordmark, version footer |
| `src-tauri/icons/*` | Replace — generate coin PNGs, .icns, .ico from `icon-zloo.svg` |
| `src-tauri/tauri.conf.json` | Modify — app name, window title |
| `src-tauri/Cargo.toml` | Modify — package name/description |
| `crates/core/Cargo.toml` | Modify — description |
| `crates/cli/Cargo.toml` | Modify — description |
| `Cargo.toml` | Modify — workspace metadata |
| `crates/cli/src/main.rs` | Modify — CLI app name/about |
| `crates/core/src/db.rs` | Modify — DB path if it references app name |
| `docs/src/**` | Modify — all doc references |
| `CLAUDE.md` | Modify — project description |
| `index.html` | Modify — `<title>` tag |

## Verification
1. `cargo build` — no compile errors with updated names
2. `npm run dev` — sidebar shows coin SVG + "złotówa" wordmark + version footer
3. Click still navigates to dashboard
4. `grep -ri "4ccountant"` returns no hits (except maybe git history)
5. Window title shows "złotówa"
6. App icon in dock/switcher shows the pixel coin (requires `cargo build` / `npm run tauri dev`)
